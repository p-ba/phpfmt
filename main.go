package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

type languageConfig struct {
	name        string
	extensions  []string
	configFiles []string
	executables []string
	defaultCmd  string
	isFormatter bool
}

var languages = []languageConfig{
	{
		name:        "python",
		extensions:  []string{".py", ".pyw"},
		configFiles: []string{".flake8", "pyproject.toml", "setup.cfg", "tox.ini", ".pylintrc", "pylintrc"},
		executables: []string{"pylint", "flake8", "black"},
		defaultCmd:  "flake8",
	},
	{
		name:        "javascript",
		extensions:  []string{".js", ".mjs", ".cjs", ".jsx"},
		configFiles: []string{".eslintrc.js", ".eslintrc.cjs", ".eslintrc.json", ".eslintrc.yml", ".eslintrc.yaml", "eslint.config.js", "prettier.config.js", ".prettierrc", ".prettierrc.json", ".prettierrc.yml", ".prettierrc.yaml"},
		executables: []string{"eslint", "prettier"},
		defaultCmd:  "eslint",
	},
	{
		name:        "typescript",
		extensions:  []string{".ts", ".tsx"},
		configFiles: []string{".eslintrc.js", ".eslintrc.cjs", ".eslintrc.json", ".eslintrc.yml", ".eslintrc.yaml", "eslint.config.js", "tsconfig.json", "prettier.config.js", ".prettierrc", ".prettierrc.json"},
		executables: []string{"eslint", "prettier"},
		defaultCmd:  "eslint",
	},
	{
		name:        "go",
		extensions:  []string{".go"},
		configFiles: []string{".golangci.yml", ".golangci.yaml", ".golangci.json", ".gofmt.toml", ".gofmt.json"},
		executables: []string{"golangci-lint", "gofmt"},
		defaultCmd:  "gofmt -w -s",
		isFormatter: true,
	},
	{
		name:        "rust",
		extensions:  []string{".rs"},
		configFiles: []string{".rustfmt.toml", "rustfmt.toml", "rust-toolchain.toml"},
		executables: []string{"rustfmt", "cargo-clippy"},
		defaultCmd:  "rustfmt",
	},
	{
		name:        "java",
		extensions:  []string{".java"},
		configFiles: []string{".clang-format", "clang-format.yaml", "clang-format.json", "google-java-format.xml"},
		executables: []string{"clang-format", "google-java-format"},
		defaultCmd:  "clang-format",
	},
	{
		name:        "php",
		extensions:  []string{".php"},
		configFiles: []string{"phpcs.xml", "phpcs.xml.dist", ".php-cs-fixer", ".php-cs-fixer.php", ".php-cs-fixer.dist", ".php-cs-fixer.dist.php"},
		executables: []string{"phpcbf", "php-cs-fixer"},
		defaultCmd:  "phpcbf --standard=PSR12",
	},
	{
		name:        "css",
		extensions:  []string{".css", ".scss", ".sass", ".less"},
		configFiles: []string{".stylelintrc", ".stylelintrc.js", ".stylelintrc.json", ".stylelintrc.yaml", ".stylelintrc.yml", "stylelint.config.js", "prettier.config.js"},
		executables: []string{"stylelint", "prettier"},
		defaultCmd:  "prettier --write",
	},
	{
		name:        "html",
		extensions:  []string{".html", ".htm"},
		configFiles: []string{".prettierrc", ".prettierrc.json", ".prettierrc.yml", "prettier.config.js"},
		executables: []string{"prettier"},
		defaultCmd:  "prettier --write",
	},
	{
		name:        "json",
		extensions:  []string{".json", ".jsonc"},
		configFiles: []string{".prettierrc", ".prettierrc.json", ".prettierrc.yaml"},
		executables: []string{"prettier"},
		defaultCmd:  "prettier --write",
	},
	{
		name:        "yaml",
		extensions:  []string{".yaml", ".yml"},
		configFiles: []string{".prettierrc", ".prettierrc.yaml", ".prettierrc.yml"},
		executables: []string{"prettier"},
		defaultCmd:  "prettier --write",
	},
	{
		name:        "markdown",
		extensions:  []string{".md", ".markdown"},
		configFiles: []string{".prettierrc", ".prettierrc.json"},
		executables: []string{"prettier"},
		defaultCmd:  "prettier --write",
	},
	{
		name:        "shell",
		extensions:  []string{".sh", ".bash", ".zsh", ".fish"},
		configFiles: []string{".shellcheckrc"},
		executables: []string{"shellcheck"},
		defaultCmd:  "shellcheck",
	},
	{
		name:        "sql",
		extensions:  []string{".sql"},
		configFiles: []string{".sqlfluff", ".sqlfluff.ini", ".sqlfluff.cfg", "pyproject.toml"},
		executables: []string{"sqlfluff"},
		defaultCmd:  "sqlfluff format",
	},
	{
		name:        "dockerfile",
		extensions:  []string{"Dockerfile", "dockerfile"},
		configFiles: []string{".hadolint.yaml", ".hadolint.yml"},
		executables: []string{"hadolint"},
		defaultCmd:  "hadolint",
	},
}

type fileBatch struct {
	config     string
	executable string
	files      []string
}

type fileBatchList struct {
	items  []fileBatch
	length int
}

func getLanguageForFile(path string) *languageConfig {
	ext := filepath.Ext(path)
	for i := range languages {
		for _, extension := range languages[i].extensions {
			if ext == extension || ext == strings.ToLower(extension) {
				return &languages[i]
			}
		}
	}
	return nil
}

func testVendor(path string, lang *languageConfig) string {
	if lang == nil {
		return ""
	}

	dir := filepath.Dir(path)
	for {
		if dir == "/" || dir == "." {
			break
		}

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

		parent := filepath.Dir(dir)
		if parent == dir {
			break
		}
		dir = parent
	}
	return ""
}

func testConfig(path string) string {
	dir := filepath.Dir(path)
	for {
		if dir == "/" || dir == "." {
			break
		}
		for _, lang := range languages {
			for _, configFile := range lang.configFiles {
				fullPath := filepath.Join(dir, configFile)
				if _, err := os.Stat(fullPath); err == nil {
					return fullPath
				}
			}
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			break
		}
		dir = parent
	}
	return ""
}

func runLinter(files, config, executable string) int {
	cmdStr := fmt.Sprintf("%s %s %s", executable, config, files)

	fmt.Printf("Executable: %s\n", executable)
	fmt.Printf("Config: %s\n", config)
	fmt.Printf("Files: %s\n", files)

	cmd := exec.Command("sh", "-c", cmdStr)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	err := cmd.Run()
	if err != nil {
		if exitError, ok := err.(*exec.ExitError); ok {
			return exitError.ExitCode()
		}
		fmt.Printf("Error running linter: %v\n", err)
		return 1 // Generic error code
	}
	return 0
}

func addToList(list *fileBatchList, file, config, executable string) {
	for i := 0; i < list.length; i++ {
		if list.items[i].config == config {
			list.items[i].files = append(list.items[i].files, file)
			return
		}
	}
	newBatch := fileBatch{
		config:     config,
		executable: executable,
		files:      []string{file},
	}
	list.items = append(list.items, newBatch)
	list.length++
}

func buildCommand(execPath, configPath string, lang *languageConfig) string {
	if execPath == "" {
		return lang.defaultCmd
	}

	baseExec := filepath.Base(execPath)

	if strings.HasPrefix(baseExec, "prettier") {
		if configPath != "" {
			return fmt.Sprintf("%s --write --config %s", execPath, configPath)
		}
		return fmt.Sprintf("%s --write", execPath)
	}

	if strings.HasPrefix(baseExec, "eslint") {
		if configPath != "" {
			return fmt.Sprintf("%s --config %s", execPath, configPath)
		}
		return fmt.Sprintf("%s", execPath)
	}

	if strings.HasPrefix(baseExec, "flake8") {
		if configPath != "" {
			return fmt.Sprintf("%s --config %s", execPath, configPath)
		}
		return fmt.Sprintf("%s", execPath)
	}

	if strings.HasPrefix(baseExec, "pylint") {
		if configPath != "" {
			return fmt.Sprintf("%s --rcfile %s", execPath, configPath)
		}
		return fmt.Sprintf("%s", execPath)
	}

	if strings.HasPrefix(baseExec, "black") {
		if configPath != "" {
			return fmt.Sprintf("%s --config %s", execPath, configPath)
		}
		return fmt.Sprintf("%s", execPath)
	}

	if strings.HasPrefix(baseExec, "gofmt") {
		if configPath != "" {
			return fmt.Sprintf("%s -w -config %s", execPath, configPath)
		}
		return fmt.Sprintf("%s -s -w", execPath)
	}

	if strings.HasPrefix(baseExec, "golangci-lint") {
		if configPath != "" {
			return fmt.Sprintf("%s run --config %s", execPath, configPath)
		}
		return fmt.Sprintf("%s run", execPath)
	}

	if strings.HasPrefix(baseExec, "rustfmt") {
		if configPath != "" {
			return fmt.Sprintf("%s --config-path %s", execPath, configPath)
		}
		return fmt.Sprintf("%s", execPath)
	}

	if strings.HasPrefix(baseExec, "cargo-clippy") {
		return fmt.Sprintf("%s", execPath)
	}

	if strings.HasPrefix(baseExec, "clang-format") {
		if configPath != "" {
			return fmt.Sprintf("%s -style=file:%s", execPath, configPath)
		}
		return fmt.Sprintf("%s", execPath)
	}

	if strings.HasPrefix(baseExec, "google-java-format") {
		if configPath != "" {
			return fmt.Sprintf("%s --aosp %s", execPath, configPath)
		}
		return fmt.Sprintf("%s", execPath)
	}

	if strings.HasPrefix(baseExec, "phpcbf") {
		if configPath != "" {
			return fmt.Sprintf("php -dmemory_limit=-1 %s --standard=%s", execPath, configPath)
		}
		return fmt.Sprintf("php -dmemory_limit=-1 %s --standard=PSR12", execPath)
	}

	if strings.HasPrefix(baseExec, "php-cs-fixer") {
		if configPath != "" {
			return fmt.Sprintf("PHP_CS_FIXER_IGNORE_ENV=true php -dmemory_limit=-1 %s fix --using-cache=no --config=%s", execPath, configPath)
		}
		return fmt.Sprintf("PHP_CS_FIXER_IGNORE_ENV=true php -dmemory_limit=-1 %s fix --rules=@Symfony,@PSR12 --using-cache=no", execPath)
	}

	if strings.HasPrefix(baseExec, "stylelint") {
		if configPath != "" {
			return fmt.Sprintf("%s --config %s", execPath, configPath)
		}
		return fmt.Sprintf("%s", execPath)
	}

	if strings.HasPrefix(baseExec, "shellcheck") {
		if configPath != "" {
			return fmt.Sprintf("%s --rcfile %s", execPath, configPath)
		}
		return fmt.Sprintf("%s", execPath)
	}

	if strings.HasPrefix(baseExec, "sqlfluff") {
		if configPath != "" {
			return fmt.Sprintf("%s format --dialect ansi --config %s", execPath, configPath)
		}
		return fmt.Sprintf("%s format --dialect ansi", execPath)
	}

	if strings.HasPrefix(baseExec, "hadolint") {
		if configPath != "" {
			return fmt.Sprintf("%s --config %s", execPath, configPath)
		}
		return fmt.Sprintf("%s", execPath)
	}

	return fmt.Sprintf("%s", execPath)
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
	fConfig := testConfig(absolutePath)
	fExecutable := testVendor(absolutePath, lang)

	if fExecutable == "" && lang != nil {
		for _, execPath := range lang.executables {
			if foundPath, err := exec.LookPath(execPath); err == nil {
				fExecutable = foundPath
				break
			}
		}
	}

	if fExecutable == "" && lang != nil {
		fExecutable = lang.defaultCmd
	}

	command := ""
	if lang != nil {
		command = buildCommand(fExecutable, fConfig, lang)
	} else {
		command = fExecutable
	}

	addToList(list, absolutePath, fConfig, command)
}

func main() {
	args := os.Args[1:]

	list := fileBatchList{
		items:  make([]fileBatch, 0),
		length: 0,
	}

	for _, arg := range args {
		walkPath(arg, &list)
	}

	if len(args) == 0 {
		walkPath(".", &list)
	}

	exitCode := 0
	for i := 0; i < list.length; i++ {
		code := runLinter(strings.Join(list.items[i].files, " "), list.items[i].config, list.items[i].executable)
		if code != 0 {
			exitCode = code
		}
	}

	os.Exit(exitCode)
}
