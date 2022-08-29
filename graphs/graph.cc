// Generates a random graph
// c++ -o graph graph.cc -O3
// ./graph
// dot -Tpng graph.dot -o graph.png
#include <cstdio>
#include <cstdlib>
#include <stdio.h>
#include <vector>

#define GRAPH_FILE "graph.dot"
#define GRAPH_OUT "graph.png"

int main(int argc, char **argv) {
  int N = 10;
  if (argc > 1) {
    N = std::atoi(argv[1]);
  }

  std::vector<std::vector<int> > adj(N);

  FILE *graph = fopen(GRAPH_FILE, "w+");

  int edges = 0.2 * N * (N - 1);
  std::fprintf(graph, "graph X {\n");
  for (int edge = 1; edge < edges; edge++) {
    int from = std::rand() % N;
    int to = std::rand() % N;

    if (from != to) {
      adj[from].push_back(to);
      adj[to].push_back(from);
      std::fprintf(graph, "\t%d -- %d\n", from, to);
    }
  }
  std::fprintf(graph, "}");
  std::fclose(graph);

  int res = std::system("neato -Tpng " GRAPH_FILE " -o " GRAPH_OUT);

  printf("Res: %d\n", res);
}
