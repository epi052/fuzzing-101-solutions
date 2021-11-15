#include <fstream>
#include <iostream>
#include <stdint.h>
#include "PDFDoc.h"
#include "goo/gtypes.h"
#include "XRef.h"

extern "C" int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
    int lastPage = 0;

    GString *user_pw = NULL;
    GString *owner_pw = NULL;
    GString *filename = NULL;

    Object obj;
    obj.initNull();

    // stream is cleaned up when doc's destructor fires
    MemStream *stream = new MemStream((char *)data, 0, size, &obj);
    PDFDoc *doc = new PDFDoc(stream, owner_pw, user_pw);

    if (doc->isOk() && doc->okToCopy()) {
        lastPage = doc->getNumPages();
    }

    if (doc) { delete doc; }

    return 0;
}