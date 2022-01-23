#include "libxml/HTMLparser.h"
#include "libxml/parser.h"
#include "libxml/tree.h"
#include "libxml/xmlversion.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#define MAXLEN 0x10000
char source[MAXLEN];

__attribute__((noinline)) int LLVMFuzzerTestOneInput(const uint8_t *data,
                                                     size_t size) {
  xmlDocPtr doc;

  int options = XML_PARSE_NOENT | XML_PARSE_DTDLOAD | XML_PARSE_DTDATTR |
                XML_PARSE_DTDVALID | XML_PARSE_HUGE | XML_PARSE_IGNORE_ENC |
                XML_PARSE_XINCLUDE | XML_PARSE_NOCDATA;

  /* xmlDocPtr	xmlReadMemory		(const char * buffer,
                                       int size,
                                       const char * URL,
                                       const char * encoding,
                                       int options)
  */
  doc = xmlReadMemory((const char *)data, size, "doesnt-matter.xml", NULL,
                      options);

  if (doc) {
    xmlFreeDoc(doc);
  }

  return 0;
}

int main(int argc, char **argv) {
  if (argc == 2) {
    FILE *fp = fopen(argv[1], "rb");
    size_t newLen = fread(source, sizeof(char), MAXLEN, fp);
    fclose(fp);
  }
  LLVMFuzzerTestOneInput((const uint8_t *)source, MAXLEN);
}
