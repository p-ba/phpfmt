package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

type commandBuilder func(execPath, configPath string) string

type languageConfig struct {
	name         string
	extensions   []string
	configFiles  []string
	executables  []string
	defaultCmd   string
	isFormatter  bool
	buildCommand commandBuilder
}

var languages = []languageConfig{
	{
		name:         "python",
		extensions:   []string{".py", ".pyw"},
		configFiles:  []string{".flake8", "pyproject.toml", "setup.cfg", "tox.ini", ".pylintrc", "pylintrc"},
		executables:  []string{"pylint", "flake8", "black"},
		defaultCmd:   "flake8",
		buildCommand: buildFlake8Cmd,
	},
	{
		name:         "javascript",
		extensions:   []string{".js", ".mjs", ".cjs", ".jsx"},
		configFiles:  []string{".eslintrc.js", ".eslintrc.cjs", ".eslintrc.json", ".eslintrc.yml", ".eslintrc.yaml", "eslint.config.js", "prettier.config.js", ".prettierrc", ".prettierrc.json", ".prettierrc.yml", ".prettierrc.yaml"},
		executables:  []string{"eslint", "prettier"},
		defaultCmd:   "eslint --fix",
		buildCommand: buildEslintCmd,
	},
	{
		name:         "typescript",
		extensions:   []string{".ts", ".tsx"},
		configFiles:  []string{".eslintrc.js", ".eslintrc.cjs", ".eslintrc.json", ".eslintrc.yml", ".eslintrc.yaml", "eslint.config.js", "prettier.config.js", ".prettierrc", ".prettierrc.json"},
		executables:  []string{"eslint", "prettier"},
		defaultCmd:   "eslint --fix",
		buildCommand: buildEslintCmd,
	},
	{
		name:         "go",
		extensions:   []string{".go"},
		configFiles:  []string{".golangci.yml", ".golangci.yaml", ".golangci.json", ".gofmt.toml", ".gofmt.json"},
		executables:  []string{"golangci-lint", "gofmt"},
		defaultCmd:   "gofmt -w -s",
		isFormatter:  true,
		buildCommand: buildGofmtCmd,
	},
	{
		name:         "rust",
		extensions:   []string{".rs"},
		configFiles:  []string{".rustfmt.toml", "rustfmt.toml", "rust-toolchain.toml"},
		executables:  []string{"rustfmt", "cargo-clippy"},
		defaultCmd:   "rustfmt",
		buildCommand: buildRustfmtCmd,
	},
	{
		name:         "java",
		extensions:   []string{".java"},
		configFiles:  []string{".clang-format", "clang-format.yaml", "clang-format.json", "google-java-format.xml"},
		executables:  []string{"clang-format", "google-java-format"},
		defaultCmd:   "clang-format",
		buildCommand: buildClangFormatCmd,
	},
	{
		name:         "php",
		extensions:   []string{".php"},
		configFiles:  []string{"phpcs.xml", "phpcs.xml.dist", ".php-cs-fixer", ".php-cs-fixer.php", ".php-cs-fixer.dist", ".php-cs-fixer.dist.php"},
		executables:  []string{"phpcbf", "php-cs-fixer"},
		defaultCmd:   "phpcbf --standard=PSR12",
		buildCommand: buildPhpcbfCmd,
	},
	{
		name:         "css",
		extensions:   []string{".css", ".scss", ".sass", ".less"},
		configFiles:  []string{".stylelintrc", ".stylelintrc.js", ".stylelintrc.json", ".stylelintrc.yaml", ".stylelintrc.yml", "stylelint.config.js", "prettier.config.js"},
		executables:  []string{"stylelint", "prettier"},
		defaultCmd:   "prettier --write",
		buildCommand: buildPrettierCmd,
	},
	{
		name:         "html",
		extensions:   []string{".html", ".htm"},
		configFiles:  []string{".prettierrc", ".prettierrc.json", ".prettierrc.yml", "prettier.config.js"},
		executables:  []string{"prettier"},
		defaultCmd:   "prettier --write",
		buildCommand: buildPrettierCmd,
	},
	{
		name:         "json",
		extensions:   []string{".json", ".jsonc"},
		configFiles:  []string{".prettierrc", ".prettierrc.json", ".prettierrc.yaml"},
		executables:  []string{"prettier"},
		defaultCmd:   "prettier --write",
		buildCommand: buildPrettierCmd,
	},
	{
		name:         "yaml",
		extensions:   []string{".yaml", ".yml"},
		configFiles:  []string{".prettierrc", ".prettierrc.yaml", ".prettierrc.yml"},
		executables:  []string{"prettier"},
		defaultCmd:   "prettier --write",
		buildCommand: buildPrettierCmd,
	},
	{
		name:         "markdown",
		extensions:   []string{".md", ".markdown"},
		configFiles:  []string{".prettierrc", ".prettierrc.json"},
		executables:  []string{"prettier"},
		defaultCmd:   "prettier --write",
		buildCommand: buildPrettierCmd,
	},
	{
		name:         "shell",
		extensions:   []string{".sh", ".bash", ".zsh", ".fish"},
		configFiles:  []string{".shellcheckrc"},
		executables:  []string{"shellcheck"},
		defaultCmd:   "shellcheck",
		buildCommand: buildShellcheckCmd,
	},
	{
		name:         "sql",
		extensions:   []string{".sql"},
		configFiles:  []string{".sqlfluff", ".sqlfluff.ini", ".sqlfluff.cfg", "pyproject.toml"},
		executables:  []string{"sqlfluff"},
		defaultCmd:   "sqlfluff format",
		buildCommand: buildSqlfluffCmd,
	},
	{
		name:         "dockerfile",
		extensions:   []string{"Dockerfile"},
		configFiles:  []string{".hadolint.yaml", ".hadolint.yml"},
		executables:  []string{"hadolint"},
		defaultCmd:   "hadolint",
		buildCommand: buildHadolintCmd,
	},
}

var extensionToLanguage = map[string]*languageConfig{}

func init() {
	for i := range languages {
		for _, ext := range languages[i].extensions {
			extensionToLanguage[ext] = &languages[i]
		}
	}
}

type fileBatch struct {
	config     string
	executable string
	files      []string
}

func buildPrettierCmd(execPath, configPath string) string {
	if configPath != "" {
		return fmt.Sprintf("%s --write --config %s", execPath, configPath)
	}
	return fmt.Sprintf("%s --write", execPath)
}

func buildEslintCmd(execPath, configPath string) string {
	if configPath != "" {
		return fmt.Sprintf("%s --fix --config %s", execPath, configPath)
	}
	return fmt.Sprintf("%s --fix", execPath)
}

func buildFlake8Cmd(execPath, configPath string) string {
	if configPath != "" {
		return fmt.Sprintf("%s --config %s", execPath, configPath)
	}
	return fmt.Sprintf("%s", execPath)
}

func buildPylintCmd(execPath, configPath string) string {
	if configPath != "" {
		return fmt.Sprintf("%s --rcfile %s", execPath, configPath)
	}
	return fmt.Sprintf("%s", execPath)
}

func buildBlackCmd(execPath, configPath string) string {
	if configPath != "" {
		return fmt.Sprintf("%s --config %s", execPath, configPath)
	}
	return fmt.Sprintf("%s", execPath)
}

func buildGofmtCmd(execPath, configPath string) string {
	if configPath != "" {
		return fmt.Sprintf("%s -w -config %s", execPath, configPath)
	}
	return fmt.Sprintf("%s -s -w", execPath)
}

func buildGolangciLintCmd(execPath, configPath string) string {
	if configPath != "" {
		return fmt.Sprintf("%s run --config %s", execPath, configPath)
	}
	return fmt.Sprintf("%s run", execPath)
}

func buildRustfmtCmd(execPath, configPath string) string {
	if configPath != "" {
		return fmt.Sprintf("%s --config-path %s", execPath, configPath)
	}
	return fmt.Sprintf("%s", execPath)
}

func buildCargoClippyCmd(execPath, configPath string) string {
	return fmt.Sprintf("%s", execPath)
}

func buildClangFormatCmd(execPath, configPath string) string {
	if configPath != "" {
		return fmt.Sprintf("%s -style=file:%s", execPath, configPath)
	}
	return fmt.Sprintf("%s", execPath)
}

func buildGoogleJavaFormatCmd(execPath, configPath string) string {
	if configPath != "" {
		return fmt.Sprintf("%s --aosp %s", execPath, configPath)
	}
	return fmt.Sprintf("%s", execPath)
}

func buildPhpcbfCmd(execPath, configPath string) string {
	if configPath != "" {
		return fmt.Sprintf("php -dmemory_limit=-1 %s --standard=%s", execPath, configPath)
	}
	return fmt.Sprintf("php -dmemory_limit=-1 %s --standard=PSR12", execPath)
}

func buildPhpCsFixerCmd(execPath, configPath string) string {
	if configPath != "" {
		return fmt.Sprintf("PHP_CS_FIXER_IGNORE_ENV=true php -dmemory_limit=-1 %s fix --using-cache=no --config=%s", execPath, configPath)
	}
	return fmt.Sprintf("PHP_CS_FIXER_IGNORE_ENV=true php -dmemory_limit=-1 %s fix --rules=@Symfony,@PSR12 --using-cache=no", execPath)
}

func buildStylelintCmd(execPath, configPath string) string {
	if configPath != "" {
		return fmt.Sprintf("%s --config %s", execPath, configPath)
	}
	return fmt.Sprintf("%s", execPath)
}

func buildShellcheckCmd(execPath, configPath string) string {
	if configPath != "" {
		return fmt.Sprintf("%s --rcfile %s", execPath, configPath)
	}
	return fmt.Sprintf("%s", execPath)
}

func buildSqlfluffCmd(execPath, configPath string) string {
	if configPath != "" {
		return fmt.Sprintf("%s format --dialect ansi --config %s", execPath, configPath)
	}
	return fmt.Sprintf("%s format --dialect ansi", execPath)
}

func buildHadolintCmd(execPath, configPath string) string {
	if configPath != "" {
		return fmt.Sprintf("%s --config %s", execPath, configPath)
	}
	return fmt.Sprintf("%s", execPath)
}

type fileBatchList struct {
	items map[string]*fileBatch
	keys  []string
}

func getLanguageForFile(path string) *languageConfig {
	ext := filepath.Ext(path)
	if lang, ok := extensionToLanguage[ext]; ok {
		return lang
	}
	if lang, ok := extensionToLanguage[strings.ToLower(ext)]; ok {
		return lang
	}
	return nil
}

func findInParentDirs(startPath string, testFunc func(string) string) string {
	dir := filepath.Dir(startPath)
	for {
		if dir == "/" || dir == "." {
			break
		}
		if result := testFunc(dir); result != "" {
			return result
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			break
		}
		dir = parent
	}
	return ""
}

func testVendor(path string, lang *languageConfig) string {
	if lang == nil {
		return ""
	}

	return findInParentDirs(path, func(dir string) string {
		if lang.name == "javascript" || lang.name == "typescript" {
			nodeModulesBinPath := filepath.Join(dir, "node_modules", ".bin")
			for _, execName := range lang.executables {
				testPath := filepath.Join(nodeModulesBinPath, execName)
				if _, err := os.Stat(testPath); err == nil {
					return testPath
				}
			}
		}

		if lang.name == "php" {
			vendorBinPath := filepath.Join(dir, "vendor", "bin")
			for _, execName := range lang.executables {
				testPath := filepath.Join(vendorBinPath, execName)
				if _, err := os.Stat(testPath); err == nil {
					return testPath
				}
			}
		}
		return ""
	})
}

func testConfig(path string, lang *languageConfig) string {
	if lang == nil {
		return ""
	}

	return findInParentDirs(path, func(dir string) string {
		for _, configFile := range lang.configFiles {
			fullPath := filepath.Join(dir, configFile)
			if _, err := os.Stat(fullPath); err == nil {
				return fullPath
			}
		}
		return ""
	})
}

func parseCommand(cmdStr string) (string, []string) {
	fields := strings.Fields(cmdStr)
	if len(fields) == 0 {
		return "", nil
	}
	return fields[0], fields[1:]
}

func runLinter(files, config, executable string) int {
	cmdStr := fmt.Sprintf("%s %s %s", executable, config, files)

	fmt.Printf("Executable: %s\n", executable)
	fmt.Printf("Config: %s\n", config)
	fmt.Printf("Files: %s\n", files)

	name, args := parseCommand(cmdStr)
	cmd := exec.Command(name, args...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	err := cmd.Run()
	if err != nil {
		if exitError, ok := err.(*exec.ExitError); ok {
			return exitError.ExitCode()
		}
		fmt.Printf("Error running linter: %v\n", err)
		return 1
	}
	return 0
}

func addToList(list *fileBatchList, file, config, executable string) {
	if list.items == nil {
		list.items = make(map[string]*fileBatch)
	}
	if batch, exists := list.items[config]; exists {
		batch.files = append(batch.files, file)
		return
	}
	newBatch := &fileBatch{
		config:     config,
		executable: executable,
		files:      []string{file},
	}
	list.items[config] = newBatch
	list.keys = append(list.keys, config)
}

func walkPath(path string, list *fileBatchList) {
	absolutePath, err := filepath.Abs(path)
	if err != nil {
		fmt.Printf("Cannot read: %s, error: %v\n", path, err)
		return
	}

	if _, err := os.Stat(absolutePath); err != nil {
		fmt.Printf("Cannot read: %s, error: %v\n", path, err)
		return
	}

	lang := getLanguageForFile(absolutePath)
	if lang == nil {
		return
	}

	fConfig := testConfig(absolutePath, lang)
	fExecutable := testVendor(absolutePath, lang)

	if fExecutable == "" {
		for _, execPath := range lang.executables {
			if foundPath, err := exec.LookPath(execPath); err == nil {
				fExecutable = foundPath
				break
			}
		}
	}

	command := lang.buildCommand(fExecutable, fConfig)
	addToList(list, absolutePath, fConfig, command)
}

func main() {
	args := os.Args[1:]

	list := fileBatchList{}

	for _, arg := range args {
		walkPath(arg, &list)
	}

	if len(args) == 0 {
		walkPath(".", &list)
	}

	exitCode := 0
	for _, key := range list.keys {
		batch := list.items[key]
		code := runLinter(strings.Join(batch.files, " "), batch.config, batch.executable)
		if code != 0 {
			exitCode = code
		}
	}

	os.Exit(exitCode)
}
