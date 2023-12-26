#include <dirent.h>
#include <errno.h>
#include <libgen.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

struct file_batch_list {
  struct file_batch *items;
  size_t length;
};

struct file_batch {
  char config[PATH_MAX];
  char executable[PATH_MAX];
  char *files;
};

void test_ptr(void *ptr) {
  if (NULL == ptr) {
    printf("Memory allocation error: %s\n", strerror(errno));
    exit(1);
  }
}

char *test_dir(char *path) {
  const char PHPCS_CONFIG_FILES[][PATH_MAX] = {"phpcs.xml", "phpcs.xml.dist"};
  const char PHP_CS_FIXER_CONFIG_FILES[][PATH_MAX] = {
      ".php-cs-fixer", ".php-cs-fixer.php", ".php-cs-fixer.dist",
      ".php-cs-fixer.dist.php"};
  char config[PATH_MAX];
  char *f_config = NULL;
  strcpy(config, path);
  for (int i = 0; i < 2; i++) {
    sprintf(config, "%s/%s", path, PHPCS_CONFIG_FILES[i]);
    if (0 == access(config, R_OK)) {
      f_config = malloc(strlen("--standard=") + strlen(path) +
                        strlen(PHPCS_CONFIG_FILES[i]) + 2);
      test_ptr(f_config);
      sprintf(f_config, "--standard=%s/%s", path, PHPCS_CONFIG_FILES[i]);
      return f_config;
    }
  }

  for (int i = 0; i < 4; i++) {
    sprintf(config, "%s/%s", path, PHP_CS_FIXER_CONFIG_FILES[i]);
    if (0 == access(config, R_OK)) {
      f_config = malloc(strlen("--config") + strlen(path) +
                        strlen(PHP_CS_FIXER_CONFIG_FILES[i]) + 2);
      test_ptr(f_config);
      sprintf(f_config, "--config=%s/%s", path, PHP_CS_FIXER_CONFIG_FILES[i]);
      return f_config;
    }
  }

  return NULL;
}

char *test_vendor(char *path) {
  char *test_phpcs = malloc(strlen(path) + strlen("/vendor/bin/phpcbf") + 1);
  strcpy(test_phpcs, path);
  strcat(test_phpcs, "/vendor/bin/phpcbf");
  if (0 == access(test_phpcs, R_OK)) {
    char *phpcs = malloc(strlen("php -dmemory_limit=-1 /vendor/bin/phpcbf") +
                         strlen(path) + 1);
    test_ptr(phpcs);
    sprintf(phpcs, "php -dmemory_limit=-1 %s/vendor/bin/phpcbf", path);
    free(test_phpcs);
    return phpcs;
  }

  char *test_php_cs_fixer =
      malloc(strlen(path) + strlen("/vendor/bin/php-cs-fixer") + 1);
  strcpy(test_php_cs_fixer, path);
  strcat(test_php_cs_fixer, "/vendor/bin/php-cs-fixer");
  if (0 == access(test_php_cs_fixer, R_OK)) {
    char *php_cs_fixer =
        malloc(strlen("PHP_CS_FIXER_IGNORE_ENV=true php -dmemory_limit=-1") +
               strlen(" fix") + strlen(test_php_cs_fixer) + 1);
    test_ptr(php_cs_fixer);
    sprintf(php_cs_fixer,
            "PHP_CS_FIXER_IGNORE_ENV=true php -dmemory_limit=-1 %s fix",
            test_php_cs_fixer);
    free(test_php_cs_fixer);
    return php_cs_fixer;
  }
  free(test_php_cs_fixer);
  free(test_phpcs);

  return NULL;
}

int run_linter(const char *file, const char *f_config,
               const char *f_executable) {
  char *cmd =
      malloc(strlen(f_executable) + strlen(f_config) + strlen(file) + 3);
  test_ptr(cmd);
  sprintf(cmd, "%s %s %s", f_executable, f_config, file);

  printf("Executable: %s\n", f_executable);
  printf("Config: %s\n", f_config);
  printf("Files: %s\n", file);
  int ret_code = system(cmd);

  free(cmd);
  return ret_code;
}

void add_new_batch(struct file_batch_list *list, char *file, char *config,
                   char *executable) {
  struct file_batch new_batch;
  new_batch.files = malloc(strlen(file) + 1);
  strcpy(new_batch.files, file);
  strcpy(new_batch.config, config);
  strcpy(new_batch.executable, executable);

  if (list->length > 0) {
    struct file_batch *copy = (struct file_batch *)realloc(
        list->items, sizeof(struct file_batch) * (list->length + 1));
    test_ptr(copy);
    copy[list->length] = new_batch;
    list->items = copy;
  } else {
    list->items = malloc(sizeof(struct file_batch));
    test_ptr(list->items);
    list->items[0] = new_batch;
  }
  list->length++;
}

void add_to_list(struct file_batch_list *list, char *file, char *config,
                 char *executable) {
  for (int i = 0; i < list->length; i++) {
    if (0 == strcmp(list->items[i].config, config)) {
      char *tmp_ptr = realloc(list->items[i].files,
                              strlen(list->items[i].files) + strlen(file) + 1);
      test_ptr(tmp_ptr);
      sprintf(tmp_ptr, "%s %s", tmp_ptr, file);
      list->items[i].files = tmp_ptr;
      return;
    }
  }
  add_new_batch(list, file, config, executable);
}

void walk_path(const char *path, struct file_batch_list *list) {
  DIR *dir;
  char *f_config = NULL, *f_executable = NULL;
  char argv_path[PATH_MAX], directory[PATH_MAX], buf[PATH_MAX];
  strcpy(argv_path, realpath(path, buf));
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
    if (NULL == f_config) {
      f_config = test_dir(directory);
    }
    if (NULL == f_executable) {
      f_executable = test_vendor(directory);
    }
    if (f_config != NULL && f_executable != NULL) {
      break;
    }
    strcat(directory, "/..");
    strcpy(directory, realpath(directory, buf));
    closedir(dir);
    dir = opendir(directory);
  }
  if (dir != NULL) {
    closedir(dir);
  }

  if (NULL == f_executable) {
    if (NULL != f_config && NULL != strstr(f_config, "phpcs")) {
      f_executable = malloc(strlen("phpcbf") + 1);
      test_ptr(f_executable);
      strcpy(f_executable, "phpcbf");
    } else {
      f_executable =
          malloc(strlen("PHP_CS_FIXER_IGNORE_ENV=true php-cs-fixer fix") + 1);
      test_ptr(f_executable);
      strcpy(f_executable, "PHP_CS_FIXER_IGNORE_ENV=true php-cs-fixer fix");
    }
  }
  if (NULL == f_config) {
    if (NULL != strstr(f_executable, "phpcbf")) {
      f_config = malloc(strlen("--standard=PSR12") + 1);
      test_ptr(f_config);
      strcpy(f_config, "--standard=PSR12");
    } else {
      f_config = malloc(strlen("--rules=@Symfony,@PSR12") + 1);
      test_ptr(f_config);
      strcpy(f_config, "--rules=@Symfony,@PSR12");
    }
  }

  add_to_list(list, argv_path, f_config, f_executable);
}

int main(int argc, const char *argv[]) {
  int exit_code = 0;
  struct file_batch_list list = {NULL, 0};
  if (1 == argc) {
    walk_path(".", &list);
  } else {
    for (int i = 1; i < argc; i++) {
      walk_path(argv[i], &list);
    }
  }

  for (int i = 0; i < list.length; i++) {
    int code = run_linter(list.items[i].files, list.items[i].config,
                          list.items[i].executable);
    free(list.items[i].files);
    if (0 != code) {
      exit_code = code;
    }
  }

  return exit_code;
}
