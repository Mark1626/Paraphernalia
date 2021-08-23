#include "ncurses.h"
#include <ncurses.h>

WINDOW *create_newwin(int height, int width, int starty, int startx);
void destroy_win(WINDOW *local_win);

int main(int argc, char** argv) {
  WINDOW *win;
  int startx, starty, width, height;
  int ch;

  initscr();
  cbreak();

  keypad(stdscr, TRUE);
  noecho();

  height = 3;
  width = 10;
  startx = (LINES - height) / 2;
  starty = (COLS - width) / 2;

  printw("Press F1 to exit");
  refresh();

  win = create_newwin(height, width, starty, startx);
  // getch();
  // destroy_win(win);
  while ((ch = getch()) != KEY_F(1)) {
    switch (ch) {
    case KEY_LEFT:
      destroy_win(win);
      win = create_newwin(height, width, starty, --startx);
      break;
    case KEY_RIGHT:
      destroy_win(win);
      win = create_newwin(height, width, starty, ++startx);
      break;
    case KEY_UP:
      destroy_win(win);
      win = create_newwin(height, width, --starty, startx);
      break;
    case KEY_DOWN:
      destroy_win(win);
      win = create_newwin(height, width, ++starty, startx);
      break;
    }
  }
  // getch();
  endwin();
  return 0;
}

WINDOW *create_newwin(int height, int width, int starty, int startx) {
  WINDOW *win = newwin(height, width, starty, startx);
  box(win, 0, 0);
  wrefresh(win);
  return win;
}

void destroy_win(WINDOW *win) {
  wborder(win, ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ');
  wrefresh(win);
  delwin(win);
}
