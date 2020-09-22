#include <ctype.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
  
#include "sexpr.h"

#define BUFFER_MAX 512

int is_float(char *str) {
  char *ptr = NULL;
  strtod(str, &ptr);
  return !*ptr;
}

int is_integer(char *str) {
  char *ptr = NULL;
  strtol(str, &ptr, 10);
  return !*ptr;
}

int is_lst_term(int c) {
  return c == EOF || isspace(c) || c == '(' || c == ')';
}

int is_str_term(int c) {
  return c == EOF || c == '"';
}

char* read_value(FILE *fp, int *c, int (*is_term)(int)) {
  int len = 0;
  char buffer[BUFFER_MAX + 1];

  while (!is_term(*c = fgetc(fp)) && len < BUFFER_MAX) {
    buffer[len] = *c;
    len++;
  }
  buffer[len] = '\0';

  char *str = malloc((len + 1) * sizeof(char));
  return strcpy(str, buffer);
}

struct SNode* snode_parse(FILE *fp) {
  struct SNode *tail, *head = NULL;
  int c;

  while ((c = fgetc(fp)) != EOF) {
    struct SNode *node = NULL;

    if (c == ')') {
      break;
    } else if (c == '(') {
      node = malloc(sizeof(struct SNode));
      node -> type = LIST;
      node -> list = snode_parse(fp);
    } else if (c == '"') { // String
      node = malloc(sizeof(struct SNode));
      node -> type = STRING;
      node -> value = read_value(fp, &c, &is_str_term);
    } else if (!isspace(c)) { // Values
      ungetc(c, fp);

      node = malloc(sizeof(struct SNode));
      node -> value = read_value(fp, &c, &is_lst_term);

      ungetc(c, fp);

      if (is_integer(node -> value)) {
        node -> type = INTEGER;
      } else if (is_float(node -> value)) {
        node -> type = FLOAT;
      } else {
        node -> type = SYMBOL;
      }
    }

    if (node != NULL) {
      node -> next = NULL;

      if (head == NULL) {
        head = tail = node;
      } else {
        tail = tail -> next = node;
      }
    }
  }

  return head;
}

void snode_free(struct SNode *node) {
  while (node != NULL) {
    struct SNode *tmp = node;

    if (node -> type == LIST) {
      snode_free(node -> list);
    } else {
      free(node -> value);
      node -> value = NULL;
    }

    node = node -> next;
    
    free(tmp);
    tmp = NULL;
  }
}

void snode_print(struct SNode *node) {
  putchar('\n');
  while (node != NULL) {
    if (node -> type == LIST) {
      printf("(\t");
      snode_print(node -> list);
      putchar(')');
    } else if (node -> type == STRING
      || node -> type == INTEGER
      || node -> type == FLOAT) {
      printf("%s\t", node -> value);
    } else if (node -> type == SYMBOL) {
      printf("%s\t", node -> value);
    }
    node = node -> next; 
  }
  putchar('\n');
}
