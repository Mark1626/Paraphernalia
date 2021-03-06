#ifndef SEXPR_H
#define SEXPR_H

enum SNodeType {
  LIST,
  STRING,
  SYMBOL,
  INTEGER,
  FLOAT
};

struct SNode {
  struct SNode *next;
  enum SNodeType type;
  union {
    struct SNode *list;
    char *value;
  };
};

struct SNode* snode_parse(FILE *fp);
void snode_free(struct SNode *node);
void snode_print(struct SNode *node);

#endif