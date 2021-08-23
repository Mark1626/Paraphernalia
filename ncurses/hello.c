#include <stdlib.h>
#include <ncurses.h>

int main() {
  initscr();
  if(has_colors() == FALSE)
	{	endwin();
		printf("Your terminal does not support color\n");
		exit(1);
	}

  init_pair(1, COLOR_RED, COLOR_BLUE);
  attron(COLOR_PAIR(1));
  printw("Hello World");
  attroff(COLOR_PAIR(1));

  refresh();
  getch();
  endwin();

  return 0;
}
