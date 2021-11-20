/**file harness.c
 * based on test-fuzzer-persistent.c
 * from test-parse.c and test-mnote.c
 *
 * Copyright (C) 2007 Hans Ulrich Niedermann <gp@n-dimensional.de>
 * Copyright 2002 Lutz Mueller <lutz@users.sourceforge.net>
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the
 * Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA  02110-1301  USA.
 */

#include <string.h>
#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/stat.h>

#include "libexif/exif-data.h"
#include "libexif/exif-loader.h"
// removed the include for "libexif/exif-system.h" because it doesn't exist in this version
//
// need to add exif-system.h's #define manually
#define UNUSED(param) UNUSED_PARAM_##param __attribute__((unused))

/** Callback function handling an ExifEntry. */
void content_foreach_func(ExifEntry *entry, void *callback_data);
void content_foreach_func(ExifEntry *entry, void *UNUSED(callback_data))
{
	char buf[2001];

	/* ensure \0 */
	buf[sizeof(buf)-1] = 0;
	buf[sizeof(buf)-2] = 0;
	exif_tag_get_name(entry->tag);
	exif_format_get_name(entry->format);
	exif_entry_get_value(entry, buf, sizeof(buf)-1);
	if (buf[sizeof(buf)-2] != 0) abort();
}


/** Callback function handling an ExifContent (corresponds 1:1 to an IFD). */
void data_foreach_func(ExifContent *content, void *callback_data);
void data_foreach_func(ExifContent *content, void *callback_data)
{
	exif_content_get_ifd(content);
	exif_content_foreach_entry(content, content_foreach_func, callback_data);
}

static int test_exif_data (ExifData *d)
{
	unsigned int i, c;
	char v[1024];
	ExifMnoteData *md;

    exif_byte_order_get_name (exif_data_get_byte_order (d));

	md = exif_data_get_mnote_data (d);
	exif_mnote_data_ref (md);
	exif_mnote_data_unref (md);

	c = exif_mnote_data_count (md);
	for (i = 0; i < c; i++) {
		const char *name = exif_mnote_data_get_name (md, i);
		if (!name) continue;
		exif_mnote_data_get_name (md, i);
		exif_mnote_data_get_title (md, i);
		exif_mnote_data_get_description (md, i);
		exif_mnote_data_get_value (md, i, v, sizeof (v));
	}

	return 0;
}

/** Main program. */
int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
	ExifData	*d;
	ExifLoader	*loader = exif_loader_new();
	unsigned int	xbuf_size;
	unsigned char	*xbuf;

    d = exif_data_new_from_data(data, size);

    /* try the exif loader */
    exif_data_foreach_content(d, data_foreach_func, NULL);
    test_exif_data (d);

    xbuf = NULL;
    exif_data_save_data (d, &xbuf, &xbuf_size);
    free (xbuf);

    exif_data_set_byte_order(d, EXIF_BYTE_ORDER_INTEL);

    xbuf = NULL;
    exif_data_save_data (d, &xbuf, &xbuf_size);
    free (xbuf);

    exif_data_unref(d);

    exif_loader_unref(loader);
	return 0;
}

#ifdef TRIAGE_TESTER
int main(int argc, char* argv[]) {
    struct stat st;
    char *filename = argv[1];

    // get file size
    stat(filename, &st);

    FILE *fd = fopen(filename, "rb");

    char *buffer = (char *)malloc(sizeof(char) * (st.st_size));

    fread(buffer, sizeof(char), st.st_size, fd);

    LLVMFuzzerTestOneInput(buffer, st.st_size);

    free(buffer);
    fclose(fd);
}
#endif