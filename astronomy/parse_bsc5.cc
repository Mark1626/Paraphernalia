#include <cstdint>
#include <cstdlib>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <ostream>
#include <string>

#if defined(__APPLE__)
#include <libkern/OSByteOrder.h>
#define htobe32(x) OSSwapHostToBigInt32(x)

#elif defined(__linux__)
#include <endian.h>
#endif

typedef std::int32_t i32;
typedef std::uint32_t u32;
typedef std::uint16_t u16;

struct __attribute__((__packed__)) SAO_Header {
  i32 star0;
  i32 star1;
  i32 starn;
  i32 stnum;
  u32 mprop;
  i32 nmag;
  i32 nbent;
};

std::ostream &operator<<(std::ostream &stream, const SAO_Header &o) {
  stream << "Header" << std::endl;
  stream << "Star0: " << o.star0 << std::endl;
  stream << "Star1: " << o.star1 << std::endl;
  stream << "Starn: " << o.starn << std::endl;
  stream << "Stnum: " << o.stnum << std::endl;
  stream << "mprop: " << o.mprop << std::endl;
  stream << "nmag: " << o.nmag << std::endl;
  stream << "nbent: " << o.nbent << std::endl;
  stream << std::endl;
  return stream;
}

struct __attribute__((__packed__)) BSC_Body {
  i32 xno;
  double sra0;
  double sdec0;
  char is[2];
  u16 mag;
  float xrpm;
  float xdpm;
};

void content_header(std::ostream &stream) {
  stream << std::setw(15) << "xno" << std::setw(15) << "sra0" << std::setw(15)
         << "sdec0" << std::setw(15) << "io" << std::setw(15) << "mag"
         << std::setw(15) << "xrpm" << std::setw(15) << "xdpm" << std::endl;
}

std::ostream &operator<<(std::ostream &stream, const BSC_Body &o) {
  stream << std::setw(15) << o.xno << std::setw(15) << o.sra0 << std::setw(15)
         << o.sdec0 << std::setw(15) << o.is[0] << o.is[1] << std::setw(15)
         << o.mag << std::setw(15) << o.xrpm << std::setw(15) << o.xdpm
         << std::endl;
  return stream;
}

const int x = sizeof(double);

class CatalogParser {
  std::string path;
  std::fstream catalog;

public:
  CatalogParser(std::string path)
      : path(path), catalog(path, std::ios::binary | std::ios::in) {
    std::cout << "Reading " << path << std::endl;
  };
  void parse_header() {

    SAO_Header header;
    catalog.read(reinterpret_cast<char *>(&header), sizeof(SAO_Header));
    std::cout << header;


    content_header(std::cout);

    for (int i = 0; i < 25; i++) {
      BSC_Body row;

      catalog.read(reinterpret_cast<char *>(&row), sizeof(BSC_Body));
      std::cout << row;
    }
  }

  void write_header() {
    std::fstream file("test.dat", std::ios::out);
    SAO_Header header = {
        .star0 = 0,
        .star1 = 1,
        .starn = 258996,
        .stnum = 1,
        .mprop = 1,
        .nmag = 1,
        .nbent = 32,
    };
    file.write(reinterpret_cast<char *>(&header), sizeof(SAO_Header));
    file.close();
  }
};

int main() {
  CatalogParser sao("./sao/SAOra.pc");
  // CatalogParser sao("test.dat");
  sao.parse_header();
  // sao.write_header();
  return EXIT_SUCCESS;
}
