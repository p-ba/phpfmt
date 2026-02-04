use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, ExitCode};

struct CommandTemplate {
    with_config: &'static str,
    without_config: &'static str,
}

impl CommandTemplate {
    const fn new(with_config: &'static str, without_config: &'static str) -> Self {
        Self {
            with_config,
            without_config,
        }
    }

    fn build(&self, exec_path: &str, config_path: &str) -> String {
        if config_path.is_empty() {
            self.without_config.replace("{exec}", exec_path)
        } else {
            self.with_config
                .replace("{exec}", exec_path)
                .replace("{config}", config_path)
        }
    }
}

const PRETTIER_TEMPLATE: CommandTemplate =
    CommandTemplate::new("{exec} --write --config {config}", "{exec} --write");

const ESLINT_TEMPLATE: CommandTemplate =
    CommandTemplate::new("{exec} --fix --config {config}", "{exec} --fix");

const SIMPLE_CONFIG_TEMPLATE: CommandTemplate =
    CommandTemplate::new("{exec} --config {config}", "{exec}");

const RCFILE_TEMPLATE: CommandTemplate = CommandTemplate::new("{exec} --rcfile {config}", "{exec}");

const GOFMT_TEMPLATE: CommandTemplate =
    CommandTemplate::new("{exec} -w -config {config}", "{exec} -s -w");

const RUSTFMT_TEMPLATE: CommandTemplate =
    CommandTemplate::new("{exec} --config-path {config}", "{exec}");

const CLANG_FORMAT_TEMPLATE: CommandTemplate =
    CommandTemplate::new("{exec} -style=file:{config}", "{exec}");

const PHPCBF_TEMPLATE: CommandTemplate = CommandTemplate::new(
    "php -dmemory_limit=-1 {exec} --standard={config}",
    "php -dmemory_limit=-1 {exec} --standard=PSR12",
);

const SQLFLUFF_TEMPLATE: CommandTemplate = CommandTemplate::new(
    "{exec} format --dialect ansi --config {config}",
    "{exec} format --dialect ansi",
);

static ESLINT_CONFIG_FILES: &[&str] = &[
    ".eslintrc.js",
    ".eslintrc.cjs",
    ".eslintrc.json",
    ".eslintrc.yml",
    ".eslintrc.yaml",
    "eslint.config.js",
];
static PRETTIER_CONFIG_FILES: &[&str] = &[
    ".prettierrc",
    ".prettierrc.json",
    ".prettierrc.yml",
    ".prettierrc.yaml",
    "prettier.config.js",
];
static STYLELINT_CONFIG_FILES: &[&str] = &[
    ".stylelintrc",
    ".stylelintrc.js",
    ".stylelintrc.json",
    ".stylelintrc.yaml",
    ".stylelintrc.yml",
    "stylelint.config.js",
];
static BLACK_CONFIG_FILES: &[&str] = &["pyproject.toml", "setup.cfg", ".flake8", "tox.ini"];
static FLAKE8_CONFIG_FILES: &[&str] = &[".flake8", "setup.cfg", "tox.ini"];
static PYLINT_CONFIG_FILES: &[&str] = &[".pylintrc", "pylintrc", "pyproject.toml"];
static GOLANGCI_CONFIG_FILES: &[&str] = &[".golangci.yml", ".golangci.yaml", ".golangci.json"];
static GOFMT_CONFIG_FILES: &[&str] = &[".gofmt.toml", ".gofmt.json"];
static RUSTFMT_CONFIG_FILES: &[&str] = &[".rustfmt.toml", "rustfmt.toml"];
static CLANG_FORMAT_CONFIG_FILES: &[&str] =
    &[".clang-format", "clang-format.yaml", "clang-format.json"];
static GOOGLE_JAVA_FORMAT_CONFIG_FILES: &[&str] = &["google-java-format.xml"];
static PHPCBF_CONFIG_FILES: &[&str] = &["phpcs.xml", "phpcs.xml.dist"];
static PHP_CS_FIXER_CONFIG_FILES: &[&str] = &[
    ".php-cs-fixer",
    ".php-cs-fixer.php",
    ".php-cs-fixer.dist",
    ".php-cs-fixer.dist.php",
];

static JS_TOOLS: &[ToolConfig] = &[
    ToolConfig::new("eslint", ESLINT_CONFIG_FILES, &ESLINT_TEMPLATE),
    ToolConfig::new("prettier", PRETTIER_CONFIG_FILES, &PRETTIER_TEMPLATE),
];
static TS_TOOLS: &[ToolConfig] = &[
    ToolConfig::new("eslint", ESLINT_CONFIG_FILES, &ESLINT_TEMPLATE),
    ToolConfig::new("prettier", PRETTIER_CONFIG_FILES, &PRETTIER_TEMPLATE),
];
static PYTHON_TOOLS: &[ToolConfig] = &[
    ToolConfig::new("black", BLACK_CONFIG_FILES, &SIMPLE_CONFIG_TEMPLATE),
    ToolConfig::new("flake8", FLAKE8_CONFIG_FILES, &SIMPLE_CONFIG_TEMPLATE),
    ToolConfig::new("pylint", PYLINT_CONFIG_FILES, &SIMPLE_CONFIG_TEMPLATE),
];
static GO_TOOLS: &[ToolConfig] = &[
    ToolConfig::new("gofmt", GOFMT_CONFIG_FILES, &GOFMT_TEMPLATE),
    ToolConfig::new(
        "golangci-lint",
        GOLANGCI_CONFIG_FILES,
        &SIMPLE_CONFIG_TEMPLATE,
    ),
];
static RUST_TOOLS: &[ToolConfig] = &[ToolConfig::new(
    "rustfmt",
    RUSTFMT_CONFIG_FILES,
    &RUSTFMT_TEMPLATE,
)];
static JAVA_TOOLS: &[ToolConfig] = &[
    ToolConfig::new(
        "clang-format",
        CLANG_FORMAT_CONFIG_FILES,
        &CLANG_FORMAT_TEMPLATE,
    ),
    ToolConfig::new(
        "google-java-format",
        GOOGLE_JAVA_FORMAT_CONFIG_FILES,
        &CLANG_FORMAT_TEMPLATE,
    ),
];
static PHP_TOOLS: &[ToolConfig] = &[
    ToolConfig::new("phpcbf", PHPCBF_CONFIG_FILES, &PHPCBF_TEMPLATE),
    ToolConfig::new(
        "php-cs-fixer",
        PHP_CS_FIXER_CONFIG_FILES,
        &SIMPLE_CONFIG_TEMPLATE,
    ),
];
static CSS_TOOLS: &[ToolConfig] = &[
    ToolConfig::new("prettier", PRETTIER_CONFIG_FILES, &PRETTIER_TEMPLATE),
    ToolConfig::new("stylelint", STYLELINT_CONFIG_FILES, &ESLINT_TEMPLATE),
];
static HTML_TOOLS: &[ToolConfig] = &[ToolConfig::new(
    "prettier",
    PRETTIER_CONFIG_FILES,
    &PRETTIER_TEMPLATE,
)];
static JSON_TOOLS: &[ToolConfig] = &[ToolConfig::new(
    "prettier",
    PRETTIER_CONFIG_FILES,
    &PRETTIER_TEMPLATE,
)];
static YAML_TOOLS: &[ToolConfig] = &[ToolConfig::new(
    "prettier",
    PRETTIER_CONFIG_FILES,
    &PRETTIER_TEMPLATE,
)];
static MARKDOWN_TOOLS: &[ToolConfig] = &[ToolConfig::new(
    "prettier",
    PRETTIER_CONFIG_FILES,
    &PRETTIER_TEMPLATE,
)];
static SHELL_TOOLS: &[ToolConfig] = &[ToolConfig::new(
    "shellcheck",
    &[".shellcheckrc"],
    &RCFILE_TEMPLATE,
)];
static SQL_TOOLS: &[ToolConfig] = &[ToolConfig::new(
    "sqlfluff",
    &[
        ".sqlfluff",
        ".sqlfluff.ini",
        ".sqlfluff.cfg",
        "pyproject.toml",
    ],
    &SQLFLUFF_TEMPLATE,
)];
static DOCKERFILE_TOOLS: &[ToolConfig] = &[ToolConfig::new(
    "hadolint",
    &[".hadolint.yaml", ".hadolint.yml"],
    &SIMPLE_CONFIG_TEMPLATE,
)];

struct ToolConfig {
    executable: &'static str,
    config_files: &'static [&'static str],
    command_template: &'static CommandTemplate,
}

impl ToolConfig {
    const fn new(
        executable: &'static str,
        config_files: &'static [&'static str],
        command_template: &'static CommandTemplate,
    ) -> Self {
        Self {
            executable,
            config_files,
            command_template,
        }
    }
}

struct LanguageConfig {
    name: &'static str,
    extensions: &'static [&'static str],
    tools: &'static [ToolConfig],
    default_cmd: &'static str,
}

struct FileBatch {
    executable: String,
    files: Vec<String>,
}

type FileBatchList = HashMap<String, FileBatch>;

static LANGUAGES: &[LanguageConfig] = &[
    LanguageConfig {
        name: "python",
        extensions: &[".py", ".pyw"],
        tools: PYTHON_TOOLS,
        default_cmd: "black",
    },
    LanguageConfig {
        name: "javascript",
        extensions: &[".js", ".mjs", ".cjs", ".jsx"],
        tools: JS_TOOLS,
        default_cmd: "eslint --fix",
    },
    LanguageConfig {
        name: "typescript",
        extensions: &[".ts", ".tsx"],
        tools: TS_TOOLS,
        default_cmd: "eslint --fix",
    },
    LanguageConfig {
        name: "go",
        extensions: &[".go"],
        tools: GO_TOOLS,
        default_cmd: "gofmt -w -s",
    },
    LanguageConfig {
        name: "rust",
        extensions: &[".rs"],
        tools: RUST_TOOLS,
        default_cmd: "rustfmt",
    },
    LanguageConfig {
        name: "java",
        extensions: &[".java"],
        tools: JAVA_TOOLS,
        default_cmd: "clang-format",
    },
    LanguageConfig {
        name: "php",
        extensions: &[".php"],
        tools: PHP_TOOLS,
        default_cmd: "phpcbf --standard=PSR12",
    },
    LanguageConfig {
        name: "css",
        extensions: &[".css", ".scss", ".sass", ".less"],
        tools: CSS_TOOLS,
        default_cmd: "prettier --write",
    },
    LanguageConfig {
        name: "html",
        extensions: &[".html", ".htm"],
        tools: HTML_TOOLS,
        default_cmd: "prettier --write",
    },
    LanguageConfig {
        name: "json",
        extensions: &[".json", ".jsonc"],
        tools: JSON_TOOLS,
        default_cmd: "prettier --write",
    },
    LanguageConfig {
        name: "yaml",
        extensions: &[".yaml", ".yml"],
        tools: YAML_TOOLS,
        default_cmd: "prettier --write",
    },
    LanguageConfig {
        name: "markdown",
        extensions: &[".md", ".markdown"],
        tools: MARKDOWN_TOOLS,
        default_cmd: "prettier --write",
    },
    LanguageConfig {
        name: "shell",
        extensions: &[".sh", ".bash", ".zsh", ".fish"],
        tools: SHELL_TOOLS,
        default_cmd: "shellcheck",
    },
    LanguageConfig {
        name: "sql",
        extensions: &[".sql"],
        tools: SQL_TOOLS,
        default_cmd: "sqlfluff format",
    },
    LanguageConfig {
        name: "dockerfile",
        extensions: &["Dockerfile"],
        tools: DOCKERFILE_TOOLS,
        default_cmd: "hadolint",
    },
];

fn get_language_for_file<'a>(
    path: &Path,
) -> Option<&'a LanguageConfig> {
    let ext = path.extension()?.to_str()?;
    let ext_with_dot = format!(".{}", ext);

    LANGUAGES.iter().find(|lang| {
        lang.extensions.iter().any(|e| {
            *e == ext
                || e.trim_start_matches('.') == ext
                || e.to_lowercase() == ext_with_dot.to_lowercase()
        })
    })
}

fn find_in_parent_dirs<F>(start_path: &Path, test_func: F) -> Option<String>
where
    F: Fn(&Path) -> Option<String>,
{
    let mut dir = start_path.parent()?;

    loop {
        if dir == Path::new("/") || dir.as_os_str() == "." {
            break;
        }

        if let Some(result) = test_func(dir) {
            return Some(result);
        }

        match dir.parent() {
            Some(parent) if parent != dir => dir = parent,
            _ => break,
        }
    }

    None
}

fn test_vendor(path: &Path, lang_name: &str, executable: &str) -> Option<String> {
    let vendor_path: &[&str] = match lang_name {
        "javascript" | "typescript" => &["node_modules", ".bin"],
        "php" => &["vendor", "bin"],
        _ => return None,
    };

    find_in_parent_dirs(path, |dir| {
        let bin_path = dir
            .join(vendor_path[0])
            .join(vendor_path[1])
            .join(executable);
        if bin_path.exists() {
            bin_path.to_str().map(String::from)
        } else {
            None
        }
    })
}

fn test_config_for_tool(path: &Path, config_files: &[&str]) -> Option<String> {
    find_in_parent_dirs(path, |dir| {
        config_files
            .iter()
            .map(|f| dir.join(f))
            .find(|p| p.exists())
            .and_then(|p| p.to_str().map(String::from))
    })
}

fn pick_tool_for_file(path: &Path, lang: &LanguageConfig) -> (&'static ToolConfig, Option<String>) {
    for tool in lang.tools {
        if let Some(config_path) = test_config_for_tool(path, tool.config_files) {
            return (tool, Some(config_path));
        }
    }
    (&lang.tools[0], None)
}

fn parse_command(cmd_str: &str) -> Option<(String, Vec<String>)> {
    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }
    Some((
        parts[0].to_string(),
        parts[1..].iter().map(|s| s.to_string()).collect(),
    ))
}

fn run_linter(files: &str, config: &str, executable: &str) -> i32 {
    let cmd_str = format!("{} {} {}", executable, config, files);

    println!("Executable: {}", executable);
    println!("Config: {}", config);
    println!("Files: {}", files);

    match parse_command(&cmd_str) {
        Some((name, args)) => Command::new(&name)
            .args(&args)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .map(|s| s.code().unwrap_or(1))
            .unwrap_or_else(|e| {
                eprintln!("Error running linter: {}", e);
                1
            }),
        None => {
            eprintln!("Error: cannot parse command");
            1
        }
    }
}

fn add_to_list(list: &mut FileBatchList, file: String, config: String, executable: String) {
    list.entry(config)
        .or_insert_with(|| FileBatch {
            executable,
            files: vec![],
        })
        .files
        .push(file);
}

fn walk_path(path: &str, list: &mut FileBatchList) {
    let absolute_path = match fs::canonicalize(path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Cannot read: {}, error: {}", path, e);
            return;
        }
    };

    if !absolute_path.exists() {
        eprintln!("Cannot read: {}, error: file not found", path);
        return;
    }

    let lang = match get_language_for_file(&absolute_path) {
        Some(l) => l,
        None => return,
    };

    let (tool, f_config_opt) = pick_tool_for_file(&absolute_path, lang);
    let f_config = f_config_opt.unwrap_or_default();

    let mut f_executable =
        test_vendor(&absolute_path, lang.name, tool.executable).unwrap_or_default();
    if f_executable.is_empty() {
        if let Ok(found) = which::which(tool.executable) {
            f_executable = found.to_string_lossy().to_string();
        }
    }
    if f_executable.is_empty() {
        f_executable = lang.default_cmd.to_string();
    }

    let command = tool.command_template.build(&f_executable, &f_config);
    add_to_list(
        list,
        absolute_path.to_string_lossy().to_string(),
        f_config,
        command,
    );
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut list: FileBatchList = HashMap::new();

    if args.is_empty() {
        walk_path(".", &mut list);
    } else {
        for arg in &args {
            walk_path(arg, &mut list);
        }
    }

    let exit_code = list
        .values()
        .map(|batch| run_linter(&batch.files.join(" "), "", &batch.executable))
        .find(|&code| code != 0)
        .unwrap_or(0);

    ExitCode::from(exit_code as u8)
}
