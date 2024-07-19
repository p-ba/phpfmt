#include <assert.h>
#include <dirent.h>
#include <errno.h>
#include <libgen.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

#define MAX_BATCH_LENGTH 100

#define MAX_PATH                                                               \
    (3 * PATH_MAX) // take more than needed to account for cli arguments

const char PHPCS_CONFIG_FILES[][MAX_PATH] = {"phpcs.xml", "phpcs.xml.dist"};
const char PHP_CS_FIXER_CONFIG_FILES[][MAX_PATH] = {
    ".php-cs-fixer", ".php-cs-fixer.php", ".php-cs-fixer.dist",
    ".php-cs-fixer.dist.php"};

struct file_batch {
    char config[MAX_PATH];
    char executable[MAX_PATH];
    char *files;
};

struct file_batch_list {
    struct file_batch items[MAX_BATCH_LENGTH];
    size_t length;
};

void test_config(char *buffer, char *path) {
    char config[MAX_PATH];

    int length = sizeof(PHPCS_CONFIG_FILES) / sizeof(PHPCS_CONFIG_FILES[0]);
    for (int i = 0; i < length; i++) {
        sprintf(config, "%s/%s", path, PHPCS_CONFIG_FILES[i]);
        if (0 == access(config, R_OK)) {
            sprintf(buffer, "--standard=%s/%s", path, PHPCS_CONFIG_FILES[i]);
            return;
        }
    }

    length = sizeof(PHP_CS_FIXER_CONFIG_FILES) / sizeof(PHP_CS_FIXER_CONFIG_FILES[0]);
    for (int i = 0; i < length; i++) {
        sprintf(config, "%s/%s", path, PHP_CS_FIXER_CONFIG_FILES[i]);
        if (0 == access(config, R_OK)) {
            sprintf(buffer, "--using-cache=no --config=%s/%s", path,
                    PHP_CS_FIXER_CONFIG_FILES[i]);
            return;
        }
    }
}

void test_vendor(char *buffer, char *path) {
    char test_phpcs[MAX_PATH];
    sprintf(test_phpcs, "%s/vendor/bin/phpcbf", path);
    if (0 == access(test_phpcs, R_OK)) {
        sprintf(buffer, "php -dmemory_limit=-1 %s/vendor/bin/phpcbf", path);
        return;
    }

    char test_php_cs_fixer[MAX_PATH];
    sprintf(test_php_cs_fixer, "%s/vendor/bin/php-cs-fixer", path);
    if (0 == access(test_php_cs_fixer, R_OK)) {
        sprintf(buffer,
                "PHP_CS_FIXER_IGNORE_ENV=true php -dmemory_limit=-1 %s fix",
                test_php_cs_fixer);
    }
}

int run_linter(const char *files, const char *f_config,
               const char *f_executable) {
    char *cmd =
        malloc(strlen(f_executable) + strlen(f_config) + strlen(files) + 3);
    assert(cmd != NULL);
    sprintf(cmd, "%s %s %s", f_executable, f_config, files);

    printf("Executable: %s\n", f_executable);
    printf("Config: %s\n", f_config);
    printf("Files: %s\n", files);
    int ret_code = system(cmd);

    free(cmd);
    return ret_code;
}

void add_new_batch(struct file_batch_list *list, char *file, char *config,
                   char *executable) {
    if (list->length + 1 == MAX_BATCH_LENGTH) {
        printf("Error: Max batch size reached");
        exit(1);
    }
    struct file_batch new_batch;
    new_batch.files = malloc(strlen(file) + 1);
    strcpy(new_batch.files, file);
    strcpy(new_batch.config, config);
    strcpy(new_batch.executable, executable);
    list->items[list->length] = new_batch;
    list->length++;
}

void add_to_list(struct file_batch_list *list, char *file, char *config,
                 char *executable) {
    for (int i = 0; i < list->length; i++) {
        if (0 == strcmp(list->items[i].config, config)) {
            char *tmp_ptr =
                realloc(list->items[i].files,
                        strlen(list->items[i].files) + strlen(file) + 1);
            assert(tmp_ptr != NULL);
            sprintf(tmp_ptr, "%s %s", tmp_ptr, file);
            list->items[i].files = tmp_ptr;
            return;
        }
    }
    add_new_batch(list, file, config, executable);
}

void walk_path(const char *path, struct file_batch_list *list) {
    DIR *dir;
    char f_config[MAX_PATH] = "\0", f_executable[MAX_PATH] = "\0";
    char argv_path[MAX_PATH] = "\0", directory[MAX_PATH] = "\0",
         buf[MAX_PATH] = "\0";
    if (NULL == realpath(path, buf)) {
        printf("Cannot read: %s, error: %s\n", path, strerror(errno));
        return;
    }
    strcpy(argv_path, buf);
    struct stat path_stat;
    stat(argv_path, &path_stat);
    if (S_ISDIR(path_stat.st_mode)) {
        strcpy(directory, argv_path);
    } else {
        strcpy(directory, dirname(argv_path));
    }
    dir = opendir(directory);
    while (dir != NULL) {
        if (strcmp(directory, "/") == 0) {
            break;
        }
        if (!strlen(f_config)) {
            test_config(f_config, directory);
        }
        if (!strlen(f_executable)) {
            test_vendor(f_executable, directory);
        }
        if (!strlen(f_config) && !strlen(f_executable)) {
            break;
        }
        strcat(directory, "/..");
        if (NULL == realpath(directory, buf)) {
            printf("Cannot read: %s, error: %s\n", path, strerror(errno));
            return;
        }
        strcpy(directory, buf);
        closedir(dir);
        dir = opendir(directory);
    }
    if (dir != NULL) {
        closedir(dir);
    }

    if (!strlen(f_executable)) {
        if (!strlen(f_config) && NULL != strstr(f_config, "phpcs")) {
            strcpy(f_executable, "phpcbf");
        } else {
            strcpy(f_executable,
                   "PHP_CS_FIXER_IGNORE_ENV=true php-cs-fixer fix");
        }
    }
    if (!strlen(f_config)) {
        if (NULL != strstr(f_executable, "phpcbf")) {
            strcpy(f_config, "--standard=PSR12");
        } else {
            strcpy(f_config, "--rules=@Symfony,@PSR12 --using-cache=no");
        }
    }

    add_to_list(list, argv_path, f_config, f_executable);
}

int main(int argc, char *argv[]) {
    unsigned int argv_length = 0;
    for (int i = 0; i < argc; i++) {
        if (strlen(argv[i]) > PATH_MAX) {
            printf("Filename is too long: %s", argv[i]);
            return 1;
        }
        argv_length += strlen(argv[i]);
    }

    struct file_batch_list *list =
        malloc((sizeof(struct file_batch_list) * argc) +
               (sizeof(struct file_batch) * argc) + argv_length);

    for (int i = 1; i < argc; i++) {
        walk_path(argv[i], list);
    }

    if (1 == argc) {
        walk_path(".", list);
    }

    int exit_code = 0;
    for (int i = 0; i < list->length; i++) {
        int code = run_linter(list->items[i].files, list->items[i].config,
                              list->items[i].executable);
        free(list->items[i].files);
        if (0 != code) {
            exit_code = code;
        }
    }
    free(list);

    return exit_code;
}
