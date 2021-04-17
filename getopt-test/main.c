#include <getopt.h>
#include <stdio.h>

static int verbose_flag;

static struct option long_option[] = {
  {.name = "name", .has_arg = required_argument, .val = 'n'},
  {.name = "verbose", .has_arg = no_argument, .flag = &verbose_flag, .val = 'v'}
};

int main(int argc, char** argv) {
  char* name;

  int option_index = 0;
  int c;
  while ((c = getopt_long(argc, argv, "n:v", long_option, &option_index)) != -1) {
    switch (c) {
      case 'n':
        name = optarg;
        break;
    }
  }

  if (verbose_flag) printf("Printing name\n");

  printf("Hello %s\n", name);

  return 0;
}

