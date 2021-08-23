#include "form.h"
#include "ncurses.h"
#include <assert.h>
#include <stdlib.h>

static FIELD *field[3];
static FORM *form;
static WINDOW *window, *window_form;

typedef struct {
  char* url;
  char* desc;
} Bookmark;

static Bookmark* book;
static int write = 0;

static void handle_input(int ch) {
  switch (ch) {
  case KEY_DOWN:
  case KEY_STAB:
  case '\t':
    form_driver(form, REQ_NEXT_FIELD);
    form_driver(form, REQ_END_LINE);
    break;
  case KEY_UP:
  case KEY_BTAB:
    form_driver(form, REQ_PREV_FIELD);
    form_driver(form, REQ_END_LINE);
    break;
  case KEY_LEFT:
    form_driver(form, REQ_PREV_CHAR);
    break;
  case KEY_RIGHT:
    form_driver(form, REQ_NEXT_CHAR);
    break;
  case KEY_HOME:
    form_driver(form, REQ_BEG_LINE);
    break;
  case KEY_END:
    form_driver(form, REQ_END_LINE);
    break;
  case KEY_BACKSPACE:
  case 127:
    form_driver(form, REQ_DEL_PREV);
    break;
  case KEY_DC:
    form_driver(form, REQ_DEL_CHAR);
  default:
    form_driver(form, ch);
    break;
  }

  wrefresh(window_form);
}

Bookmark* read_form() {
  Bookmark* temp = (Bookmark*) malloc(sizeof(Bookmark) * 1);
  form_driver(form, REQ_NEXT_FIELD);
  form_driver(form, REQ_PREV_FIELD);
  temp->url = field_buffer(field[0], 0);
  temp->desc = field_buffer(field[1], 0);
  return temp;
}

void print_book() {
  if (book) {
    printf("Url: %s \n", book->url);
    printf("Desc: %s \n", book->desc);
  }
}

void init_curses() {
  initscr();
  cbreak();
  noecho();
  keypad(stdscr, TRUE);

  window = newwin(24, 80, 0, 0);
  window_form = derwin(window, 20, 78, 3, 1);
  assert(window != NULL);
  box(window_form, 0, 0);
  mvwprintw(window_form, 1, 2, "Press Esc to quit");
}

void init_form() {
  field[0] = new_field(1, 10, 4, 18, 0, 0);
  field[1] = new_field(1, 10, 6, 18, 0, 0);
  field[2] = NULL;
  assert(field[0] != NULL && field[1] != NULL);

  set_field_back(field[0], A_UNDERLINE);
  field_opts_off(field[0], O_AUTOSKIP);

  set_field_back(field[1], A_UNDERLINE);
  field_opts_off(field[1], O_AUTOSKIP);

  form = new_form(field);
  assert(form != NULL);
  set_form_win(form, window_form);
  set_form_sub(form, derwin(window_form, 18, 76, 1, 1));
  post_form(form);

  mvwprintw(window_form, 4, 10, "Url:");
  mvwprintw(window_form, 6, 10, "Desc:");
  refresh();

  wrefresh(window);
  wrefresh(window_form);
}

void deinit_form() {
  unpost_form(form);
  free_form(form);
  free_field(field[0]);
  free_field(field[1]);
}

void deinit() {
  delwin(window_form);
  delwin(window);
  endwin();
}

int main() {
  int ch;

  init_curses();
  init_form();

  ch = getch();
  while (ch != KEY_F(1) && ch != KEY_F(2)) {
    handle_input(ch);
    ch = getch();
  }

  if (ch == KEY_F(2)) {
    book = read_form();
  }

  print_book();

  if (book != NULL) {
    free(book);
  }

  deinit_form();
  deinit();

  return 0;
}
