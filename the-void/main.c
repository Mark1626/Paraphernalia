#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <getopt.h>
#include <sqlite3.h>

#define FILE_SIZE_LIMIT 268435456

void print_usage() {
  printf("Expecting void [flags] -f <file>\n");
  printf("\nSupported flags:\n");
  printf("    -f           name of file\n");
  printf("    -v           verbose\n");
  printf("    -h           show this message\n");
}

int parse_args(int argc, char** argv, char* fileName, int* verbose) {
  if (argc < 2) {
    print_usage();
    return EXIT_FAILURE;
  }

  int c;
  while((c = getopt(argc, argv, "f:vh")) != -1) {
    switch(c) {
      case 'f':
        strcpy(fileName, optarg);
      break;
      case 'v':
        *verbose = 1;
      break;
      case '?':
        if (optopt == 'f') {
          fprintf(stderr, "File not given\n");
        }
        return EXIT_FAILURE;
        break;
      case 'h':
      default:
        print_usage();
      break;
    }
  }
  return EXIT_SUCCESS;
}

int write_blob_to_db(char* fileName, char* blob, int blob_size) {
  sqlite3* db;
  int rc = sqlite3_open("./test.db", &db);
  if (rc) {
    sqlite3_close(db);
    return EXIT_FAILURE;
  }

  char query[] = "insert into files values (?, ?)";
  sqlite3_stmt *stmt;

  rc = sqlite3_prepare_v2(db, query, -1, &stmt, 0);

  if (rc == SQLITE_OK) {
    sqlite3_bind_text(stmt, 1, fileName, strlen(query), NULL);
    sqlite3_bind_text(stmt, 2, blob, blob_size, NULL);
  } else {
    fprintf(stderr, "Failed to execute statement: %s\n", sqlite3_errmsg(db));
  }

  rc = sqlite3_step(stmt);

  if (rc == SQLITE_ROW) {
    printf("%s\n", sqlite3_column_text(stmt, 0));
    printf("%s\n", sqlite3_column_text(stmt, 1));
  }

  sqlite3_finalize(stmt);
  sqlite3_close(db);
  return EXIT_SUCCESS;
}

int store_file_as_blob(char *file_name) {
  char* buffer;
  int rc;

  FILE *fp = fopen(file_name, "r");
  if (fp == NULL) {
    fprintf(stderr, "Error reading file\n");
    return EXIT_FAILURE;
  }

  // Find file size
  fseek(fp, 0, SEEK_END);
  long lSize = ftell(fp);
  rewind(fp);

  // Read the content into the buffer
  if (lSize > FILE_SIZE_LIMIT) {
    fprintf(stderr, "File size is too large\n");
    rc = EXIT_FAILURE;
  } else {
    buffer = malloc(sizeof(char) * lSize);
    if (buffer == NULL) {
      fputs("Error creating buffer", stderr);
      return EXIT_FAILURE;
    }

    rc = fread(buffer, 1, lSize, fp);
    if (rc != lSize) {
      fprintf(stderr, "Error reading content of file\n");
      return EXIT_FAILURE;
    }

    rc = write_blob_to_db(file_name, buffer, lSize);
    if (rc) {
      fprintf(stderr, "Error writing to db\n");
    }
    free(buffer);
  }

  fclose(fp);
  return rc;
}

int main(int argc, char** argv) {
  char file_name[512];
  int verbose = 0;
  int rc = parse_args(argc, argv, file_name, &verbose);

  if (rc) {
    fprintf(stderr, "Error parsing arguments\n");
    return EXIT_FAILURE;
  }

  store_file_as_blob(file_name);

  return EXIT_SUCCESS;
}


