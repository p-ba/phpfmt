package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

var phpcsFixerConfigFiels = []string{
	".php-cs-fixer",
	".php-cs-fixer.php",
	".php-cs-fixer.dist",
	".php-cs-fixer.dist.php",
}

var phpcsConfigFiles = []string{
	"phpcs.xml",
	"phpcs.xml.dist",
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

func testVendor(path string) string {
	testPhpcs := filepath.Join(path, "vendor", "bin", "phpcbf")
	if _, err := os.Stat(testPhpcs); err == nil {
		return fmt.Sprintf("php -dmemory_limit=-1 %s", testPhpcs)
	}

	testPhpCsFixer := filepath.Join(path, "vendor", "bin", "php-cs-fixer")
	if _, err := os.Stat(testPhpCsFixer); err == nil {
		return fmt.Sprintf("PHP_CS_FIXER_IGNORE_ENV=true php -dmemory_limit=-1 %s fix", testPhpCsFixer)
	}
	return ""
}

func testConfig(path string) string {
	for _, configFile := range phpcsConfigFiles {
		var config = filepath.Join(path, configFile)
		if _, err := os.Stat(config); err == nil {
			return fmt.Sprintf("--standard=%s", config)
		}
	}
	for _, configFile := range phpcsFixerConfigFiels {
		var config = filepath.Join(path, configFile)
		if _, err := os.Stat(config); err == nil {
			return fmt.Sprintf("--using-cache=no --config=%s", config)
		}
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

func walkPath(path string, list *fileBatchList) {
	var fConfig, fExecutable string

	absolutePath, err := filepath.Abs(path)
	if err != nil {
		fmt.Printf("Cannot read: %s, error: %v\n", path, err)
		return
	}

	fileInfo, err := os.Stat(absolutePath)
	if err != nil {
		fmt.Printf("Cannot read: %s, error: %v\n", path, err)
		return
	}

	directory := absolutePath
	if !fileInfo.IsDir() {
		directory = filepath.Dir(absolutePath)
	}

	for {
		if directory == "/" {
			break
		}

		if fConfig == "" {
			fConfig = testConfig(directory)
		}

		if fExecutable == "" {
			fExecutable = testVendor(directory)
		}

		if fConfig != "" && fExecutable != "" {
			break
		}

		directory = filepath.Join(directory, "..")
		directory, err = filepath.Abs(directory)
		if err != nil {
			fmt.Printf("Cannot read: %s, error: %v\n", path, err)
			return
		}
	}

	if fExecutable == "" {
		if fConfig != "" && strings.Contains(fConfig, "phpcs") {
			fExecutable = "phpcbf"
		} else {
			fExecutable = "PHP_CS_FIXER_IGNORE_ENV=true php-cs-fixer fix"
		}
	}

	if fConfig == "" {
		if strings.Contains(fExecutable, "phpcbf") {
			fConfig = "--standard=PSR12"
		} else {
			fConfig = "--rules=@Symfony,@PSR12 --using-cache=no"
		}
	}

	addToList(list, absolutePath, fConfig, fExecutable)
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
