use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, ExitCode};

struct CommandTemplate {
    with_config: &'static [&'static str],
    without_config: &'static [&'static str],
}

impl CommandTemplate {
    const fn new(
        with_config: &'static [&'static str],
        without_config: &'static [&'static str],
    ) -> Self {
        Self {
            with_config,
            without_config,
        }
    }

    fn build(&self, exec_path: &str, config_path: Option<&str>) -> Vec<String> {
        let selected_template = if config_path.is_some() {
            self.with_config
        } else {
            self.without_config
        };

        selected_template
            .iter()
            .map(|part| {
                let mut resolved = part.replace("{exec}", exec_path);
                if let Some(config) = config_path {
                    resolved = resolved.replace("{config}", config);
                }
                resolved
            })
            .collect()
    }
}

const PRETTIER_TEMPLATE: CommandTemplate = CommandTemplate::new(
    &["{exec}", "--write", "--config", "{config}"],
    &["{exec}", "--write"],
);

const ESLINT_TEMPLATE: CommandTemplate = CommandTemplate::new(
    &["{exec}", "--fix", "--config", "{config}"],
    &["{exec}", "--fix"],
);

const SIMPLE_CONFIG_TEMPLATE: CommandTemplate =
    CommandTemplate::new(&["{exec}", "--config", "{config}"], &["{exec}"]);

const RCFILE_TEMPLATE: CommandTemplate =
    CommandTemplate::new(&["{exec}", "--rcfile", "{config}"], &["{exec}"]);

const GOFMT_TEMPLATE: CommandTemplate = CommandTemplate::new(
    &["{exec}", "-w", "-config", "{config}"],
    &["{exec}", "-s", "-w"],
);

const RUSTFMT_TEMPLATE: CommandTemplate =
    CommandTemplate::new(&["{exec}", "--config-path", "{config}"], &["{exec}"]);

const CLANG_FORMAT_TEMPLATE: CommandTemplate =
    CommandTemplate::new(&["{exec}", "-style=file:{config}"], &["{exec}"]);

const PHPCBF_TEMPLATE: CommandTemplate = CommandTemplate::new(
    &["php", "-dmemory_limit=-1", "{exec}", "--standard={config}"],
    &["php", "-dmemory_limit=-1", "{exec}", "--standard=PSR12"],
);

const SQLFLUFF_TEMPLATE: CommandTemplate = CommandTemplate::new(
    &[
        "{exec}",
        "format",
        "--dialect",
        "ansi",
        "--config",
        "{config}",
    ],
    &["{exec}", "format", "--dialect", "ansi"],
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

static JS_TS_TOOLS: &[ToolConfig] = &[
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
static PRETTIER_ONLY_TOOLS: &[ToolConfig] = &[ToolConfig::new(
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
    default_executable: &'static str,
}

impl LanguageConfig {
    const fn new(
        name: &'static str,
        extensions: &'static [&'static str],
        tools: &'static [ToolConfig],
        default_executable: &'static str,
    ) -> Self {
        Self {
            name,
            extensions,
            tools,
            default_executable,
        }
    }
}

struct FileBatch {
    program: String,
    args: Vec<String>,
    files: Vec<String>,
}

type FileBatchList = HashMap<String, FileBatch>;

static LANGUAGES: &[LanguageConfig] = &[
    LanguageConfig::new("python", &[".py", ".pyw"], PYTHON_TOOLS, "black"),
    LanguageConfig::new(
        "javascript",
        &[".js", ".mjs", ".cjs", ".jsx"],
        JS_TS_TOOLS,
        "eslint",
    ),
    LanguageConfig::new("typescript", &[".ts", ".tsx"], JS_TS_TOOLS, "eslint"),
    LanguageConfig::new("go", &[".go"], GO_TOOLS, "gofmt"),
    LanguageConfig::new("rust", &[".rs"], RUST_TOOLS, "rustfmt"),
    LanguageConfig::new("java", &[".java"], JAVA_TOOLS, "clang-format"),
    LanguageConfig::new("php", &[".php"], PHP_TOOLS, "phpcbf"),
    LanguageConfig::new(
        "css",
        &[".css", ".scss", ".sass", ".less"],
        CSS_TOOLS,
        "prettier",
    ),
    LanguageConfig::new("html", &[".html", ".htm"], PRETTIER_ONLY_TOOLS, "prettier"),
    LanguageConfig::new(
        "json",
        &[".json", ".jsonc"],
        PRETTIER_ONLY_TOOLS,
        "prettier",
    ),
    LanguageConfig::new("yaml", &[".yaml", ".yml"], PRETTIER_ONLY_TOOLS, "prettier"),
    LanguageConfig::new(
        "markdown",
        &[".md", ".markdown"],
        PRETTIER_ONLY_TOOLS,
        "prettier",
    ),
    LanguageConfig::new(
        "shell",
        &[".sh", ".bash", ".zsh", ".fish"],
        SHELL_TOOLS,
        "shellcheck",
    ),
    LanguageConfig::new("sql", &[".sql"], SQL_TOOLS, "sqlfluff"),
    LanguageConfig::new("dockerfile", &["Dockerfile"], DOCKERFILE_TOOLS, "hadolint"),
];

fn get_language_for_file<'a>(path: &Path) -> Option<&'a LanguageConfig> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .or_else(|| path.file_name().and_then(|n| n.to_str()))?;
    let ext_normalized = ext.trim_start_matches('.').to_lowercase();

    LANGUAGES.iter().find(|lang| {
        lang.extensions
            .iter()
            .any(|e| e.trim_start_matches('.').to_lowercase() == ext_normalized)
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
        "javascript" | "typescript" | "css" | "html" | "json" | "yaml" | "markdown" => {
            &["node_modules", ".bin"]
        }
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

fn resolve_executable(path: &Path, lang: &LanguageConfig, tool: &ToolConfig) -> String {
    test_vendor(path, lang.name, tool.executable)
        .or_else(|| {
            which::which(tool.executable)
                .ok()
                .map(|p| p.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| lang.default_executable.to_string())
}

fn pick_tool_for_file(path: &Path, lang: &LanguageConfig) -> (&'static ToolConfig, Option<String>) {
    for tool in lang.tools {
        if let Some(config_path) = test_config_for_tool(path, tool.config_files) {
            return (tool, Some(config_path));
        }
    }
    (&lang.tools[0], None)
}

fn build_command(
    executable: &str,
    _lang: &LanguageConfig,
    tool: &ToolConfig,
    config_path: Option<&str>,
) -> Option<(String, Vec<String>)> {
    let args = tool.command_template.build(executable, config_path);
    if args.is_empty() {
        return None;
    }

    let mut parts = args.into_iter();
    let program = parts.next()?;
    Some((program, parts.collect()))
}

fn is_executable_available(program: &str) -> bool {
    let program_path = Path::new(program);

    if program_path.is_absolute() || program.contains('/') || program.contains('\\') {
        return program_path.exists();
    }

    which::which(program).is_ok()
}

fn run_linter(files: &[String], program: &str, args: &[String]) -> i32 {
    println!("Program: {}", program);
    println!("Args: {:?}", args);
    println!("Files: {}", files.join(" "));

    if !is_executable_available(program) {
        eprintln!(
            "Cannot run formatter/linter: '{}' is not installed or not on PATH",
            program
        );
        return 1;
    }

    Command::new(program)
        .args(args)
        .args(files)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map(|s| s.code().unwrap_or(1))
        .unwrap_or_else(|e| {
            eprintln!("Error running linter: {}", e);
            1
        })
}

fn add_to_list(list: &mut FileBatchList, file: String, program: String, args: Vec<String>) {
    let key = format!("{}\0{}", program, args.join("\0"));

    list.entry(key)
        .or_insert_with(|| FileBatch {
            program,
            args,
            files: vec![],
        })
        .files
        .push(file);
}

fn process_file(path: &Path, list: &mut FileBatchList) {
    let absolute_path = match fs::canonicalize(path) {
        Ok(p) if p.is_file() => p,
        Err(e) => {
            eprintln!("Cannot read: {}, error: {}", path.display(), e);
            return;
        }
        _ => return,
    };

    let Some(lang) = get_language_for_file(&absolute_path) else {
        return;
    };

    let (tool, config_path) = pick_tool_for_file(&absolute_path, lang);
    let executable = resolve_executable(&absolute_path, lang, tool);
    let Some((program, args)) = build_command(&executable, lang, tool, config_path.as_deref())
    else {
        eprintln!(
            "Cannot build command for {}",
            absolute_path.to_string_lossy()
        );
        return;
    };

    add_to_list(
        list,
        absolute_path.to_string_lossy().to_string(),
        program,
        args,
    );
}

fn should_skip_dir(name: &str) -> bool {
    matches!(
        name,
        ".git" | "node_modules" | "target" | "vendor" | ".venv" | "venv"
    )
}

fn walk_entry(path: &Path, list: &mut FileBatchList) {
    if path.is_file() {
        process_file(path, list);
        return;
    }

    if !path.is_dir() {
        return;
    }

    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Cannot read directory: {}, error: {}", path.display(), e);
            return;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Cannot read directory entry in {}: {}", path.display(), e);
                continue;
            }
        };

        let entry_path = entry.path();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(e) => {
                eprintln!("Cannot read file type for {}: {}", entry_path.display(), e);
                continue;
            }
        };

        if file_type.is_symlink() {
            continue;
        }

        if file_type.is_dir()
            && entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(should_skip_dir)
        {
            continue;
        }

        walk_entry(&entry_path, list);
    }
}

fn walk_path(path: &str, list: &mut FileBatchList) {
    let input_path = Path::new(path);

    if !input_path.exists() {
        eprintln!("Cannot read: {}, error: file not found", path);
        return;
    }

    walk_entry(input_path, list);
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
        .map(|batch| run_linter(&batch.files, &batch.program, &batch.args))
        .find(|&code| code != 0)
        .unwrap_or(0);

    ExitCode::from(exit_code as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn builds_command_with_default_args_when_tool_is_missing() {
        let lang = &LANGUAGES[1]; // javascript
        let tool = &lang.tools[0]; // eslint
        let (program, args) = build_command("eslint", lang, tool, None).expect("command");

        assert_eq!(program, "eslint");
        assert_eq!(args, vec!["--fix".to_string()]);
    }

    #[test]
    fn keeps_batches_separate_for_different_commands() {
        let mut list: FileBatchList = HashMap::new();
        add_to_list(
            &mut list,
            "one.rs".to_string(),
            "rustfmt".to_string(),
            vec![],
        );
        add_to_list(
            &mut list,
            "two.ts".to_string(),
            "eslint".to_string(),
            vec!["--fix".to_string()],
        );

        assert_eq!(list.len(), 2);
    }

    #[test]
    fn detects_dockerfile_by_file_name() {
        let dockerfile = PathBuf::from("Dockerfile");
        let lang = get_language_for_file(&dockerfile).expect("dockerfile language");

        assert_eq!(lang.name, "dockerfile");
    }
}
