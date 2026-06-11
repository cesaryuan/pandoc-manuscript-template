- - ## Getting started
		- ## Using MathType
		- ## Technical documentation
		- ## MathType 7
			- ## MathType SDK
				- ## Getting started to MathType's API
								- ## MathType SDK documentation
					- ## Application-specific metafile comment convention
										- ## Converting equations
										- ## EGO (Edit Graphic Object) specification
										- ## Embedded data in PDF
										- ## Expanding MathType's font and character information
										- ## Extracting baseline info from a GIF file
										- ## Extracting baseline info from a Mac PICT
										- ## Extracting baseline info from an EPS file
										- ## Extracting baseline info from a Windows Metafile
										- ## How MathML is stored in files and the clipboard
										- ## How MTEF is stored in files and objects
										- ## Interpreting the baseline
										- ## MathPage batch processing
										- ## MathPage MathML Targets
										- ## MathPage Settings
										- ## MathType MTEF v.3 (Equation Editor 3.x)
										- ## MathType MTEF v.5 (MathType 4.0 and later)
										- ## Using.NET to access MathType's OLE subsystem
										- ## Using MFC to access MathType's OLE subsystem
										- ## Using MTSDKDN
										- ## MathType MTEF v.4 (MathType 3.5)
										- ## Translator programmers manual
						- ## Network administrator's manual
						- ## MathType's command line options
				- ## MathType for Microsoft 365
				- ## MathType for HTML editors
				- ## MathType for LMS
		- ## Troubleshooting & FAQs
		- ## Reference and legal
- ## WirisQuizzes
- ## Nubric
- ## CalcMe
- ## MathPlayer
- ## Store FAQ
- ## MathFlow
- ## BF FAQ
- ## Miscellaneous
- ## Wiris Integrations

## Abstract

This document describes, the binary equation format used by MathType 4.0 and later (all platforms). is embedded in OLE equation objects produced by MathType as well as in all the file formats in which MathType can save equations. The methods used by MathType to embed this information in such files are described in a separate document. See [How is Stored in Files and Objects](https://docs.wiris.com/mathtype/en/mathtype-office-tools/mathtype-7-for-windows-and-mac/mathtype-sdk/MTEFstorage).

In addition to this online documentation, a consolidated package containing the same documentation, along with source code examples, can be provided as a single file upon request. Please contact [support@wiris.com](mailto:support@wiris.com), including your organization or affiliation and a brief explanation of the intended use of the package.

## Introduction

This document describes the binary equation format used by MathType 4.0 and later (all platforms). Although is not the most friendly medium for defining equations, there have been so many requests for this information, we decided to publish it anyway. We must warn the reader that it is not an easy format to understand and, more importantly, MathType is not at all forgiving in its processing of it. This means that if you send MathType with errors, it might crash. At a minimum, you will get an equation with formatting problems. Also, it is a binary format. This means that you can't use character strings to represent equations and it makes creating a little harder with programming languages like Visual Basic.

How MathType stores an equation description in an OLE equation object, a file, or on the clipboard is not described here. Please see the document on [MathType Storage](https://docs.wiris.com/mathtype/en/mathtype-office-tools/mathtype-7-for-windows-and-mac/mathtype-sdk/MTEFstorage) for more information on this subject.

This document sometimes refers to MathType internal names for values (e.g. parmLINESPACE). These are given for reference purposes and are handy for reducing errors when such values are communicated by humans.

## Header

### version history:

data exists in the following versions:

| 0 | MathType for Mac 1.x (this format is not described here) |
| --- | --- |
| 1 | MathType for Mac 2.x and MathType for Windows 1.x |
| 2 | MathType 3.x and Equation Editor 1.x |
| 3 | Equation Editor 3.x (this format is not described here) |
| 4 | MathType 3.5 |
| 5 | MathType 4.0 and later |

Version 2 differs from version 1 only in the format of the header.

### Header record:

The version 5 header contains:

| length in bytes | description | value |
| --- | --- | --- |
| 1 | version | 5 |
| 1 | generating platform | 0 for Mac, 1 for Windows |
| 1 | generating product | 0 for MathType, 1 for Equation Editor |
| 1 | product version | 4 or later |
| 1 | product subversion | 0 (or???) |
| length of app key | application key | null-terminated string identifying the writing application (e.g. "DSMT4" for the Wiris version of MathType) |
| 1 | equation options | if bit 0 is set, equation is inline, else display equation other bits are unused and must be 0 |

## Byte Stream

This section describes the actual data following the header.

### Records:

data consists of a series of records. Each record starts with a record type byte and an options byte, then is followed by data specific to the record.

The overall structure of an stream is:

Once the header has been read, reading of the stream should be driven by reading the next record type and then acting on it. If a given record depends on data that is not contained within the record itself, such data is defined by records that precede it.

### Record types:

The following record types are used:

| value | symbol | description |
| --- | --- | --- |
| 0 | [END](https://docs.wiris.com/mathtype/en/) | end of, pile, line, embellishment list, or template |
| 1 | [LINE](https://docs.wiris.com/mathtype/en/) | line (slot) |
| 2 | [CHAR](https://docs.wiris.com/mathtype/en/) | character |
| 3 | [TMPL](https://docs.wiris.com/mathtype/en/) | template |
| 4 | [PILE](https://docs.wiris.com/mathtype/en/) | pile (vertical stack of lines) |
| 5 | [MATRIX](https://docs.wiris.com/mathtype/en/) | matrix |
| 6 | [EMBELL](https://docs.wiris.com/mathtype/en/) | character embellishment (e.g. hat, prime) |
| 7 | [RULER](https://docs.wiris.com/mathtype/en/) | ruler (tab-stop location) |
| 8 | [FONT\_STYLE\_DEF](https://docs.wiris.com/mathtype/en/) | font/char style definition |
| 9 | [SIZE](https://docs.wiris.com/mathtype/en/) | general size |
| 10 | [FULL](https://docs.wiris.com/mathtype/en/) | full size |
| 11 | [SUB](https://docs.wiris.com/mathtype/en/) | subscript size |
| 12 | [SUB2](https://docs.wiris.com/mathtype/en/) | sub-subscript size |
| 13 | [SYM](https://docs.wiris.com/mathtype/en/) | symbol size |
| 14 | [SUBSYM](https://docs.wiris.com/mathtype/en/) | sub-symbol size |
| 15 | [COLOR](https://docs.wiris.com/mathtype/en/) | color |
| 16 | [COLOR\_DEF](https://docs.wiris.com/mathtype/en/) | color definition |
| 17 | [FONT\_DEF](https://docs.wiris.com/mathtype/en/) | font definition |
| 18 | [EQN\_PREFS](https://docs.wiris.com/mathtype/en/) | equation preferences (sizes, styles, spacing) |
| 19 | [ENCODING\_DEF](https://docs.wiris.com/mathtype/en/) | encoding definition |
| ≥ 100 | FUTURE | for future expansion (see below) |

If the record type is 100 or greater, it represents a record that will be defined in a future version of. For now, readers can assume that an [unsigned integer](https://docs.wiris.com/mathtype/en/) follows the record type and is the number of bytes following it in the record (i.e. it doesn't include the record type and length). This makes it easy for software that reads to skip these records. Although it might be handy if all records had such a length value, it will only be present on future expansion records (i.e. those with record types ≥ 100).

### Object lists:

[LINE](https://docs.wiris.com/mathtype/en/), [CHAR](https://docs.wiris.com/mathtype/en/), [TMPL](https://docs.wiris.com/mathtype/en/), [PILE](https://docs.wiris.com/mathtype/en/), [MATRIX](https://docs.wiris.com/mathtype/en/), and [RULER](https://docs.wiris.com/mathtype/en/) records are followed by object lists that define contents of each equation structure. Each object list contains a sequence of records of any type and terminated by an [END](https://docs.wiris.com/mathtype/en/) record. In a special case for [LINE](https://docs.wiris.com/mathtype/en/) records, if there are no objects in the list, the line record will have the [OPT\_LINE\_NULL](https://docs.wiris.com/mathtype/en/) option set, in which case the object list is omitted entirely (i.e. no [END](https://docs.wiris.com/mathtype/en/) record). Although there are no restrictions made by the format on what record types may occur in any particular list, the user interface prevents certain things from happening. For example, the object list defining the contents of a pile contains only [LINE](https://docs.wiris.com/mathtype/en/) records.

### Definition records:

Some of 's records ([FONT\_STYLE\_DEF](https://docs.wiris.com/mathtype/en/), [FONT\_DEF](https://docs.wiris.com/mathtype/en/), [COLOR\_DEF](https://docs.wiris.com/mathtype/en/), and [ENCODING\_DEF](https://docs.wiris.com/mathtype/en/)) do not represent equation structure themselves but are referred to by equation structure records. The definition records in an stream are assigned indices, starting at 1 (except for [ENCODING\_DEF](https://docs.wiris.com/mathtype/en/) s, which start at 5), in the order they appear in the stream. Records which refer to these definitions do so using the index value of the definition. Definition records always appear in the stream before their first reference.

### Option values:

Each 5 record starts with a type byte followed by an option byte. This is different from earlier versions of where the option flags were stored in the upper 4 bits of the type byte.

The option flag values are record-dependent:

| value | symbol | description |
| --- | --- | --- |
| Option flag values for all equation structure records: |  |  |
| 0×08 | OPT\_NUDGE | [nudge values](https://docs.wiris.com/mathtype/en/) follow tag |
| Option flag values for [CHAR](https://docs.wiris.com/mathtype/en/) records: |  |  |
| 0×01 | OPT\_CHAR\_EMBELL | character is followed by an embellishment list |
| 0×02 | OPT\_CHAR\_FUNC\_START | character starts a function (sin, cos, etc.) |
| 0×04 | OPT\_CHAR\_ENC\_CHAR\_8 | character is written with an 8-bit encoded value |
| 0×10 | OPT\_CHAR\_ENC\_CHAR\_16 | character is written with an 16-bit encoded value |
| 0×20 | OPT\_CHAR\_ENC\_NO\_MTCODE | character is written without an 16-bit MTCode value |
| Option flag values for [LINE](https://docs.wiris.com/mathtype/en/) records: |  |  |
| 0×01 | OPT\_LINE\_NULL | line is a placeholder only (i.e. not displayed) |
| 0×04 | OPT\_LINE\_LSPACE | line spacing value follows tag |
| Option flag values for [LINE](https://docs.wiris.com/mathtype/en/) and [PILE](https://docs.wiris.com/mathtype/en/) records: |  |  |
| 0×02 | OPT\_LP\_RULER | [RULER](https://docs.wiris.com/mathtype/en/) record follows [LINE](https://docs.wiris.com/mathtype/en/) or [PILE](https://docs.wiris.com/mathtype/en/) record |
| Option flag values for [COLOR\_DEF](https://docs.wiris.com/mathtype/en/) records: |  |  |
| 0×01 | COLOR\_CMYK | color model is CMYK, else RGB |
| 0×02 | COLOR\_SPOT | color is a spot color, else a process color |
| 0×04 | COLOR\_NAME | color has a name, else no name |

### Dimensional units:

All dimensional values are expressed in MathType internal units, 32nds of a printer's point (a point is 1/72 inch).

### Signed integer values:

Signed values are written as follows:

| range of value | algorithm |
| --- | --- |
| \-128 ≤ value < 127 | value = value + 128 written as a single byte |
| value < -128 or value ≥ 127 | value = value + 32768 written as 3 bytes: byte 1: 255 byte 2: low byte of value byte 3: high byte of value |

### Unsigned integer values:

Unsigned values are written as follows:

| range of value | algorithm |
| --- | --- |
| value < 255 | written as a single byte |
| value ≥ 255 | written as 3 bytes: byte 1: 255 byte 2: low byte of value byte 3: high byte of value |

### Simple 16-bit integer values:

Some values are written as 16-bits even if the value would fit in a byte. In this case, the value is written low byte followed by high byte.

### Nudge values:

[LINE](https://docs.wiris.com/mathtype/en/), [CHAR](https://docs.wiris.com/mathtype/en/), [TMPL](https://docs.wiris.com/mathtype/en/), [PILE](https://docs.wiris.com/mathtype/en/), [MATRIX](https://docs.wiris.com/mathtype/en/), and [EMBELL](https://docs.wiris.com/mathtype/en/) records may store the result of nudging (small offsets applied by the user). A nudged record has the OPT\_NUDGE option (0×8) and the option byte is followed immediately by the nudge offset. The nudge offset consists of either two bytes or six, depending on the amount of offset. If -128 ≤ dx < +128 and -128 ≤ dy < +128, then the offsets are stored as two bytes, dx followed by dy, where each value has 128 added to it before it is written. Otherwise, two bytes of 128 are stored, followed by the offsets, dx and dy, stored as 16-bit values, low byte followed by high byte.

### Typeface values:

[CHAR](https://docs.wiris.com/mathtype/en/) records contain a typeface value (biased by 128), written as a [signed integer](https://docs.wiris.com/mathtype/en/). If the value is positive, it represents one of MathType styles:

| value | symbol |
| --- | --- |
| 1 | fnTEXT |
| 2 | fnFUNCTION |
| 3 | fnVARIABLE |
| 4 | fnLCGREEK |
| 5 | fnUCGREEK |
| 6 | fnSYMBOL |
| 7 | fnVECTOR |
| 8 | fnNUMBER |
| 9 | fnUSER1 |
| 10 | fnUSER2 |
| 11 | fnMTEXTRA |
| 12 | fnTEXT\_FE |
| 22 | fnEXPAND |
| 23 | fnMARKER |
| 24 | fnSPACE |

If the value is negative, it represents an explicit font as specified by a FONT record.

### Typesize values:

Typesize values (sometimes referred to as lsizes) are used in several records. Not all values may be valid in a particular record. Their meaning is as follows:

| value | symbol | description |
| --- | --- | --- |
| 0 | szFULL full |  |
| 1 | szSUB subscript |  |
| 2 | szSUB2 sub-subscript |  |
| 3 | szSYM symbol |  |
| 4 | szSUBSYM sub-symbol |  |
| 5 | szUSER1 user 1 |  |
| 6 | szUSER2 user 2 |  |
| 7 | szDELTA delta increment |  |

### Character style values:

Character styles are represented by a single byte, bit 0 indicates bold and bit 1 indicates italic. In other words:

| 0 | plain |
| --- | --- |
| 1 | bold |
| 2 | italic |
| 3 | bold and italic |

### Horizontal alignment values:

Horizontal alignment values are used in several records. Not all values may be valid in a particular record. Their meaning is as follows:

| 1 | left justification |
| --- | --- |
| 2 | centered |
| 3 | right justification |
| 4 | relational operator alignment |
| 5 | decimal point alignment |

### Vertical alignment values:

Vertical alignment values are used in several records. Not all values may be valid in a particular record. Their meaning is as follows:

| 0 | alignment with baseline of top line |
| --- | --- |
| 1 | alignment with baseline of center line |
| 2 | alignment with baseline of bottom line |
| 3 | vertical centering 4 alignment with the math axis (center of +,-, brace points, etc.) |

### Dimension arrays:

In the [EQN\_PREFS](https://docs.wiris.com/mathtype/en/) record, sizes and spacing values are both written as dimension arrays. These arrays are used to record the particular settings from the Define Sizes and Define Spacing dialogs used to define the equation. Instead of recording each value as a number, the dimension array captures each value as a character string. This ensures the user always sees the value just as entered, rather than possibly modified by rounding and/or truncation in conversion (e.g. so "2.0 inches" doesn't turn into "1.999 inches").

Each array is written as a single byte count of dimensions in the array, followed by a "nibble stream" containing the dimensions. Each nibble is a 4-bit value, packed 2 per byte. The upper 4 bits of each byte precedes the lower 4 bits. If the array contains an odd number of nibbles, an additional 0 nibble is written to round out the stream to whole bytes.

Each dimension in the array consists of a units nibble (see first table below), followed by a nibble for each character in the value string (see second table below), terminated by a 0×F nibble. The following table shows how each units nibble is interpreted:

| 0 | inches |
| --- | --- |
| 1 | centimeters |
| 2 | points |
| 3 | picas |
| 4 | percentage |

The following table shows how each nibble in a value string is interpreted:

| 0×0-0×9 | decimal digit |
| --- | --- |
| 0×A | decimal point |
| 0×B | minus sign |
| 0×F | end of value string |

## Record Details

The following details the individual records in. All values are single bytes unless indicated otherwise.

### END record (0):

Consists of:

- [record type](https://docs.wiris.com/mathtype/en/) (0)

There is no option byte.

### LINE record (1):

Consists of:

The line spacing value, if present, is the distance between the baseline of this line and the line above it.

### CHAR record (2):

Consists of:

The character value itself is represented by one or more values. The presence or absence of these value is indicated by options and appear in this order:

| [16-bit integer](https://docs.wiris.com/mathtype/en/) MTCode value | present unless the OPT\_CHAR\_ENC\_NO\_MTCODE option is set |
| --- | --- |
| 8-bit font position | present if the OPT\_CHAR\_ENC\_CHAR\_8 option is set |
| [16-bit integer](https://docs.wiris.com/mathtype/en/) font position | present if the OPT\_CHAR\_ENC\_CHAR\_16 option is set |

The MTCode value defines the character independent of its font. MTCode is a superset of Unicode and is described in [MTCode Encoding Tables](http://www.dessci.com/support/tech/encodings/mtcode.stm). The 8-bit and 16-bit font positions are mutually exclusive but may both be absent. This is the position of the character within its font. Some of the common font encodings are given in [Font Encoding Tables](http://www.dessci.com/support/tech/encodings/font_enc.stm).

### TMPL record (3):

Consists of:

- [record type](https://docs.wiris.com/mathtype/en/) (3)
- [options](https://docs.wiris.com/mathtype/en/mathtype-office-tools/support/mathtype-tips---tricks/tips-to-use-with-microsoft-powerpoint.html#options)
- \[[nudge](https://docs.wiris.com/mathtype/en/)\] if OPT\_NUDGE is set
- \[[selector](https://docs.wiris.com/mathtype/en/)\] template selector code
- \[[variation](https://docs.wiris.com/mathtype/en/)\] template variation code (1 or 2 bytes; see below)
- \[options\] template-specific options
- \[subobject list\] either a single character (e.g. sigma in a summation) and/or lines

The template selector and variation codes determine the class of the template and various properties of the template, such as which subobjects can be deleted by the user (see [Templates](https://docs.wiris.com/mathtype/en/mathtype-office-tools/mathtype-7-for-windows-and-mac/mathtype-sdk/mathtype-mtef-v-5--mathtype-4-0-and-later-.html#templates-2923252)). The class of a template determines the order and meaning of each of its subobjects (see [Template subobject order](https://docs.wiris.com/mathtype/en/)).

The variation code may be 1 or 2 bytes long. If the first byte value has the high bit set (0×80), the next byte is read and combined with the first according to this formula:

variation code = (byte1 & 0×7F) | (byte2 << 8)

The template-specific options field is only used for integrals and fence templates:

| Fence template option field values (fence alignment): |  |
| --- | --- |
| 0 | center fence on math axis, place math axis of contents on math axis of containing line (default); |
| 1 | center fence on contents, place math axis of contents on math axis of containing line; |
| 2 | center fence on contents, center contents on math axis of containing line. |
| Warning: the expanding integral property is duplicated in the integral templates variation codes (see Limit variations). On reading, MathType only looks at the variation code. |  |
| Integral template option field values: |  |
| 0 | fixed-size integral; |
| 1 | the integral expands vertically to fit its contents. |

### PILE record (4):

Consists of:

### MATRIX record (5):

Consists of:

- [record type](https://docs.wiris.com/mathtype/en/) (5)
- [options](https://docs.wiris.com/mathtype/en/mathtype-office-tools/support/mathtype-tips---tricks/tips-to-use-with-microsoft-powerpoint.html#options)
- \[[nudge](https://docs.wiris.com/mathtype/en/)\] if OPT\_NUDGE is set
- \[[valign](https://docs.wiris.com/mathtype/en/)\] vertical alignment of matrix within container
- \[[h\_just](https://docs.wiris.com/mathtype/en/)\] horizontal alignment within columns
- \[[v\_just](https://docs.wiris.com/mathtype/en/)\] vertical alignment within columns
- \[rows\] number of rows
- \[cols\] number of columns
- \[row\_parts\] row partition line types (see below)
- \[col\_parts\] column partition line types (see below)
- \[[object list](https://docs.wiris.com/mathtype/en/)\] list of [lines](https://docs.wiris.com/mathtype/en/), one for each element of the matrix, in order from left-to-right and top-to-bottom

The values for valign, h\_just, and v\_just are described in [PILE](https://docs.wiris.com/mathtype/en/) above.

The row partition line type list consists of two-bit values for each possible partition line (one more than the number of rows), rounded out to the nearest byte. Each value determines the line style of the corresponding partition line (0 for none, 1 for solid, 2 for dashed, or 3 for dotted). Similarly for the column partition lines.

### EMBELL record (6):

Consists of:

- [record type](https://docs.wiris.com/mathtype/en/) (6)
- [options](https://docs.wiris.com/mathtype/en/mathtype-office-tools/support/mathtype-tips---tricks/tips-to-use-with-microsoft-powerpoint.html#options)
- \[[nudge](https://docs.wiris.com/mathtype/en/)\] if OPT\_NUDGE is set
- \[embell\] embellishment type

The embellishment types are:

| value | symbol | description |
| --- | --- | --- |
| 2 | emb1DOT | over single dot |
| 3 | emb2DOT | over double dot |
| 4 | emb3DOT | over triple dot |
| 5 | emb1PRIME | single prime |
| 6 | emb2PRIME | double prime |
| 7 | embBPRIME | backwards prime (left of character) |
| 8 | embTILDE | tilde |
| 9 | embHAT | hat (circumflex) |
| 10 | embNOT | diagonal slash through character |
| 11 | embRARROW | over right arrow |
| 12 | embLARROW | over left arrow |
| 13 | embBARROW | over both arrow (left and right) |
| 14 | embR1ARROW | over right single-barbed arrow |
| 15 | embL1ARROW | over left single-barbed arrow |
| 16 | embMBAR | mid-height horizontal bar |
| 17 | embOBAR | over-bar |
| 18 | emb3PRIME | triple prime |
| 19 | embFROWN | over-arc, concave downward |
| 20 | embSMILE | over-arc, concave upward |
| 21 | embX\_BARS | double diagonal bars |
| 22 | embUP\_BAR | bottom-left to top-right diagonal bar |
| 23 | embDOWN\_BAR | top-left to bottom-right diagonal bar |
| 24 | emb4DOT | over quad dot |
| 25 | embU\_1DOT | under single dot |
| 26 | embU\_2DOT | under double dot |
| 27 | embU\_3DOT | under triple dot |
| 28 | embU\_4DOT | under quad dot |
| 29 | embU\_BAR | under bar |
| 30 | embU\_TILDE | under tilde (~) |
| 31 | embU\_FROWN | under arc (ends point down) |
| 32 | embU\_SMILE | under arc (ends point up) |
| 33 | embU\_RARROW | under right arrow |
| 34 | embU\_LARROW | under left arrow |
| 35 | embU\_BARROW | under both arrow (left and right) |
| 36 | embU\_R1ARROW | under right arrow (1 barb) |
| 37 | embU\_L1ARROW | under left arrow (1 barb) |

### RULER record (7):

Consists of:

- [record type](https://docs.wiris.com/mathtype/en/) (7)
- \[n\_stops\] number of tab-stops
- \[tab-stop list\] tab-stops in order from left-to-right

Each tab stop is described by a tab-stop type (0 for left, 1 for center, 2 for right, 3 for equal, 4 for decimal), followed by a [16-bit integer](https://docs.wiris.com/mathtype/en/) offset from the left end of the slot or pile with which it is associated.

### SIZE record (9):

Consists of one of the following cases:

if lsize < 0 (explicit point size):

else: (large delta)

- [record type](https://docs.wiris.com/mathtype/en/) (9)
- 100 lsize ([typesize](https://docs.wiris.com/mathtype/en/mathtype-office-tools/appendix/glossary.html#typesizes))
- dsize ([16 bit integer](https://docs.wiris.com/mathtype/en/))

Sizes in MathType are represented as a pair of values, lsize and dsize. Lsize stands for "logical size", dsize for "delta size". If it is negative, it is an explicit point size (in 32nds of a point) negated and dsize is ignored. Otherwise, lsize is a [typesize](https://docs.wiris.com/mathtype/en/mathtype-office-tools/appendix/glossary.html#typesizes) value and dsize is a delta from that size:

Simple typesizes, without a delta value, are written using the records described in the next section.

### Typesize records (10-14):

Consists of:

- [record type](https://docs.wiris.com/mathtype/en/) (10-14)

These records are just short ways of specifying a simple typesize where dsize is zero. The tag value represents an lsize + 10. So if the tag value is 10, it means equation content following it will be Full size (szFULL), tag value 11 means szSUB, and so on. See [typesize](https://docs.wiris.com/mathtype/en/mathtype-office-tools/appendix/glossary.html#typesizes).

### COLOR records (15):

Consists of:

- [record type](https://docs.wiris.com/mathtype/en/) (15)
- \[color\_def\_index\] index of corresponding [COLOR\_DEF](https://docs.wiris.com/mathtype/en/) record ([unsigned integer](https://docs.wiris.com/mathtype/en/))

The appearance of this record in the stream indicates that all following equation records (until the next COLOR record) have the color defined by the indicated [COLOR\_DEF](https://docs.wiris.com/mathtype/en/) record.

### COLOR\_DEF records (16):

Consists of:

- [record type](https://docs.wiris.com/mathtype/en/) (16)
- \[[options](https://docs.wiris.com/mathtype/en/mathtype-office-tools/support/mathtype-tips---tricks/tips-to-use-with-microsoft-powerpoint.html#options)\] model is RGB unless COLOR\_CMYK bit is set; type is process unless COLOR\_SPOT bit is set; color is unnamed unless COLOR\_NAME bit is set
- \[color values\] if RGB, 3 values (red, green, blue); if CMYK, 4 values (cyan, magenta, yellow, black); see below for details
- \[name\] null-terminated color name; appears only if COLOR\_NAME option is set

This record defines a color (see [Definition records](https://docs.wiris.com/mathtype/en/)). Each color value is written as a [16-bit integer](https://docs.wiris.com/mathtype/en/) that ranges between 0 and 1000 where 0 is the absence of the color and 1000 is a fully saturated color. So, an RGB color definition for black has all three components at 0.

### FONT\_DEF records (17):

Consists of:

- [record type](https://docs.wiris.com/mathtype/en/) (17)
- \[enc\_def\_index\] index of corresponding ENCODING\_DEF record ([unsigned integer](https://docs.wiris.com/mathtype/en/))
- \[name\] null-terminated font name

This record associates an font encoding with a font name. See [Definition records](https://docs.wiris.com/mathtype/en/).

### EQN\_PREFS records (18):

Consists of:

When reading arrays, the number of values may be less than or greater than expected. readers should be driven by the array count. If the array is shorter than expected, assume defaults for the missing values. If the array is longer than expected, the extra values must be skipped to stay in sync with the stream.

Spacing values are written in the following order:

| 0 | parmLINESPACE | Line spacing |
| --- | --- | --- |
| 1 | parmMATRIXROW | Matrix row spacing |
| 2 | parmMATRIXCOL | Matrix column spacing |
| 3 | parmSUPHEIGHT | Superscript height |
| 4 | parmSUBDEPTH | Subscript depth |
| 5 | parmSUBSUPGAP | Sub/superscript gap |
| 6 | parmLIMHEIGHT | Limit height |
| 7 | parmLIMDEPTH | Limit depth |
| 8 | parmLIMSPACE | Limit line spacing |
| 9 | parmFRACTHEIGHT | Numerator height |
| 10 | parmFRACTDEPTH | Denominator depth |
| 11 | parmFRACTOVER | Fraction bar overhang |
| 12 | parmFRACTTHICK | Fraction bar thickness |
| 13 | parmFRACTTHICK2 | Sub-fraction bar thickness |
| 14 | parmFRACTGAP | Slash/diagonal fraction gap |
| 15 | parmFENCEOVER | Fence overhang |
| 16 | parmOPERSPACING | Operator spacing (% of normal) |
| 17 | parmNONOPERSPACING | Non-operator spacing (% of normal) |
| 18 | parmCHARWIDTH | Character width adjustment |
| 19 | parmMINGAP | Minimum gap |
| 20 | parmVRADGAP | Radical gap (vertical) |
| 21 | parmHRADGAP | Radical gap (horizontal) |
| 22 | parmRADWIDTH | Radical width (% of normal) |
| 23 | parmEMBELLGAP | Embellishment gap |
| 24 | parmPRIMEHEIGHT | Prime Height |
| 25 | parmBOX\_STROKE\_THICK | Box stroke thickness |
| 26 | parmSTRIKE\_THRU\_THICK | Strike-through thickness |
| 27 | parmMATRIX\_PART\_THICK | Matrix partition line thickness |
| 28 | parmRAD\_THICK | Radical stroke thickness |
| 29 | parmHORIZ\_FENCE\_GAP | Horizontal fence gap |

The style definition array is written as a single byte count followed by that number of style definitions. The order is defined by [Typeface values](https://docs.wiris.com/mathtype/en/), however only fnTEXT through fnTEXT\_FE are written. Each style definition is written as an unsigned integer that is 0 if the style is unused in the equation or is the index of the corresponding [FONT\_DEF](https://docs.wiris.com/mathtype/en/) record. If the style is used (not 0), it is followed by a single byte [character style](https://docs.wiris.com/mathtype/en/).

### ENCODING\_DEF records (19):

Consists of:

- [record type](https://docs.wiris.com/mathtype/en/) (19)
- \[name\] null-terminated encoding name

This record defines (see [Definition records](https://docs.wiris.com/mathtype/en/)) a font encoding and is referred to by a [FONT\_DEF](https://docs.wiris.com/mathtype/en/) record. In order to reduce the size of the stream, the following 4 encodings are predefined:

| ENCODING\_DEF index | encoding name |
| --- | --- |
| 1 | [MTCode](http://www.dessci.com/en/support/mathtype/tech/encodings/mtcode.htm) |
| 2 | Unknown |
| 3 | [Symbol](http://www.dessci.com/en/support/mathtype/tech/encodings/symbol.htm) |
| 4 | [MTExtra](http://www.dessci.com/en/support/mathtype/tech/encodings/mtextra.htm) |

This means that the first ENCODING\_DEF record in the stream is considered to have an index of 5. See [Extending MathType's font and character information](https://docs.wiris.com/mathtype/en/mathtype-office-tools/mathtype-7-for-windows-and-mac/mathtype-sdk#extending_mathtype_s_font_and_character_information) and [MathType's character encodings](https://docs.wiris.com/mathtype/en/mathtype-office-tools/mathtype-7-for-windows-and-mac/mathtype-sdk#mathtype_s_character_encodings) for more information on font encodings.

## Templates

This section shows the selector and variation codes for all the templates. The class names can be used to determine the order of subobjects in the list following the template tag (see [Template subobject order](https://docs.wiris.com/mathtype/en/)).

### Limit variations:

The following variation codes apply to all templates whose class is BigOpBoxClass or LimBoxClass:

| variation bits | symbol | description |
| --- | --- | --- |
| 0×0001 | tvBO\_LOWER | lower limit is present |
| 0×0002 | tvBO\_UPPER | upper limit is present |
| 0×0040 | tvBO\_SUM | summation-style limit positions, else integral-style |

### Template selectors and variations:

<table><colgroup><col> <col> <col> <col></colgroup><thead><tr><th>Fences (parentheses, etc.):</th><th></th><th></th><th></th></tr></thead><tbody><tr><td><strong>selector</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td><td><strong>class</strong></td></tr><tr><td>0</td><td>tmANGLE</td><td>angle brackets</td><td><a href="https://docs.wiris.com/mathtype/en/">ParBoxClass</a></td></tr><tr><td>1</td><td>tmPAREN</td><td>parentheses</td><td><a href="https://docs.wiris.com/mathtype/en/">ParBoxClass</a></td></tr><tr><td>2</td><td>tmBRACE</td><td>braces (curly brackets)</td><td><a href="https://docs.wiris.com/mathtype/en/">ParBoxClass</a></td></tr><tr><td>3</td><td>tmBRACK</td><td>square brackets</td><td><a href="https://docs.wiris.com/mathtype/en/">ParBoxClass</a></td></tr><tr><td>4</td><td>tmBAR</td><td>vertical bars</td><td><a href="https://docs.wiris.com/mathtype/en/">ParBoxClass</a></td></tr><tr><td>5</td><td>tmDBAR</td><td>double vertical bars</td><td><a href="https://docs.wiris.com/mathtype/en/">ParBoxClass</a></td></tr><tr><td>6</td><td>tmFLOOR</td><td>floor brackets</td><td><a href="https://docs.wiris.com/mathtype/en/">ParBoxClass</a></td></tr><tr><td>7</td><td>tmCEILING</td><td>ceiling brackets</td><td><a href="https://docs.wiris.com/mathtype/en/">ParBoxClass</a></td></tr><tr><td>8</td><td>tmOBRACK</td><td>open (white) brackets</td><td><a href="https://docs.wiris.com/mathtype/en/">ParBoxClass</a></td></tr><tr><td><strong>variations</strong></td><td><strong>variation bits</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td></tr><tr><td>:::</td><td>0×0001</td><td>tvFENCE_L</td><td>left fence is present</td></tr><tr><td>:::</td><td>0×0002</td><td>tvFENCE_R</td><td>right fence is present</td></tr><tr><td colspan="4">Intervals:</wrap></td></tr><tr><td><strong>selector</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td><td><strong>class</strong></td></tr><tr><td>9</td><td>tmINTERVAL</td><td>unmatched brackets and parentheses</td><td><a href="https://docs.wiris.com/mathtype/en/">ParBoxClass</a></td></tr><tr><td><strong>variations</strong></td><td><strong>variation bits</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td></tr><tr><td>:::</td><td>0×0000</td><td>tvINTV_LEFT_LP</td><td>left fence is left parenthesis</td></tr><tr><td>:::</td><td>0×0001</td><td>tvINTV_LEFT_RP</td><td>left fence is right parenthesis</td></tr><tr><td>:::</td><td>0×0002</td><td>tvINTV_LEFT_LB</td><td>left fence is left bracket</td></tr><tr><td>:::</td><td>0×0003</td><td>tvINTV_LEFT_RB</td><td>left fence is right bracket</td></tr><tr><td>:::</td><td>0×0000</td><td>tvINTV_RIGHT_LP</td><td>right fence is left parenthesis</td></tr><tr><td>:::</td><td>0×0010</td><td>tvINTV_RIGHT_RP</td><td>right fence is right parenthesis</td></tr><tr><td>:::</td><td>0×0020</td><td>tvINTV_RIGHT_LB</td><td>right fence is left bracket</td></tr><tr><td>:::</td><td>0×0030</td><td>tvINTV_RIGHT_RB</td><td>right fence is right bracket</td></tr><tr><td colspan="4">Radicals (square and nth roots):</td></tr><tr><td><strong>selector</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td><td><strong>class</strong></td></tr><tr><td>10</td><td>tmROOT</td><td>radical</td><td><a href="https://docs.wiris.com/mathtype/en/">RootBoxClass</a></td></tr><tr><td><strong>variations</strong></td><td><strong>variation</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td></tr><tr><td>:::</td><td>0</td><td>tvROOT_SQ</td><td>square root</td></tr><tr><td>:::</td><td>1</td><td>tvROOT_NTH</td><td>nth root</td></tr><tr><td colspan="4">Fractions:</wrap></td></tr><tr><td><strong>selector</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td><td><strong>class</strong></td></tr><tr><td>11</td><td>tmFRACT</td><td>fractions</td><td><a href="https://docs.wiris.com/mathtype/en/">FracBoxClass</a></td></tr><tr><td><strong>variations</strong></td><td><strong>variation bits</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td></tr><tr><td>:::</td><td>0×0001</td><td>tvFR_SMALL</td><td>subscript-size slots (piece fraction)</td></tr><tr><td>:::</td><td>0×0002</td><td>tvFR_SLASH</td><td>fraction bar is a slash</td></tr><tr><td>:::</td><td>0×0004</td><td>tvFR_BASE</td><td>num. and denom. are baseline aligned</td></tr><tr><td colspan="4">Over and Underbars:</td></tr><tr><td><strong>selector</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td><td><strong>class</strong></td></tr><tr><td>12</td><td>tmUBAR</td><td>underbar</td><td>BarBoxClass</td></tr><tr><td>13</td><td>tmOBAR</td><td>overbar</td><td>BarBoxClass</td></tr><tr><td><strong>variations</strong></td><td><strong>variation bits</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td></tr><tr><td>:::</td><td>0×0001</td><td>tvBAR_DOUBLE</td><td>bar is doubled, else single</td></tr><tr><td colspan="4">Arrows:</wrap></td></tr><tr><td><strong>selector</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td><td><strong>class</strong></td></tr><tr><td>14</td><td>tmARROW</td><td>arrow</td><td><a href="https://docs.wiris.com/mathtype/en/">ArroBoxClass</a></td></tr><tr><td><strong>variations</strong></td><td><strong>variation</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td></tr><tr><td>:::</td><td>0×0000</td><td>tvAR_SINGLE</td><td>single arrow</td></tr><tr><td>:::</td><td>0×0001</td><td>tvAR_DOUBLE</td><td>double arrow</td></tr><tr><td>:::</td><td>0×0002</td><td>tvAR_HARPOON</td><td>harpoon</td></tr><tr><td>:::</td><td>0×0004</td><td>tvAR_TOP</td><td>top slot is present</td></tr><tr><td>:::</td><td>0×0008</td><td>tvAR_BOTTOM</td><td>bottom slot is present</td></tr><tr><td>:::</td><td>0×0010</td><td>tvAR_LEFT</td><td>if single, arrow points left</td></tr><tr><td>:::</td><td>0×0020</td><td>tvAR_RIGHT</td><td>if single, arrow points right</td></tr><tr><td>:::</td><td>0×0010</td><td>tvAR_LOS</td><td>if double or harpoon, large over small</td></tr><tr><td>:::</td><td>0×0020</td><td>tvAR_SOL</td><td>if double or harpoon, small over large</td></tr><tr><td colspan="4">Integrals (see <a href="https://docs.wiris.com/mathtype/en/">Limit Variations</a>):</td></tr><tr><td><strong>selector</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td><td><strong>class</strong></td></tr><tr><td>15</td><td>tmINTEG</td><td>integral</td><td><a href="https://docs.wiris.com/mathtype/en/">BigOpBoxClass</a></td></tr><tr><td><strong>variations</strong></td><td><strong>variation</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td></tr><tr><td>:::</td><td>0×0001</td><td>tvINT_1</td><td>single integral sign</td></tr><tr><td>:::</td><td>0×0002</td><td>tvINT_2</td><td>double integral sign</td></tr><tr><td>:::</td><td>0×0003</td><td>tvINT_3</td><td>triple integral sign</td></tr><tr><td>:::</td><td>0×0004</td><td>tvINT_LOOP</td><td>has loop w/o arrows</td></tr><tr><td>:::</td><td>0×0008</td><td>tvINT_CW_LOOP</td><td>has clockwise loop</td></tr><tr><td>:::</td><td>0×000C</td><td>tvINT_CCW_LOOP</td><td>has counter-clockwise loop</td></tr><tr><td>:::</td><td>0×0100</td><td>tvINT_EXPAND</td><td>integral signs expand</td></tr><tr><td colspan="4">Sums, products, coproducts, unions, intersections, etc. (see <a href="https://docs.wiris.com/mathtype/en/">Limit Variations</a>):</td></tr><tr><td><strong>selector</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td><td><strong>class</strong></td></tr><tr><td>16</td><td>tmSUM</td><td>sum</td><td><a href="https://docs.wiris.com/mathtype/en/">BigOpBoxClass</a></td></tr><tr><td>17</td><td>tmPROD</td><td>product</td><td><a href="https://docs.wiris.com/mathtype/en/">BigOpBoxClass</a></td></tr><tr><td>18</td><td>tmCOPROD</td><td>coproduct</td><td><a href="https://docs.wiris.com/mathtype/en/">BigOpBoxClass</a></td></tr><tr><td>19</td><td>tmUNION</td><td>union</td><td><a href="https://docs.wiris.com/mathtype/en/">BigOpBoxClass</a></td></tr><tr><td>20</td><td>tmINTER</td><td>intersection</td><td><a href="https://docs.wiris.com/mathtype/en/">BigOpBoxClass</a></td></tr><tr><td>21</td><td>tmINTOP</td><td>integral-style big operator</td><td><a href="https://docs.wiris.com/mathtype/en/">BigOpBoxClass</a></td></tr><tr><td>22</td><td>tmSUMOP</td><td>summation-style big operator</td><td><a href="https://docs.wiris.com/mathtype/en/">BigOpBoxClass</a></td></tr><tr><td colspan="4">Limits (see <a href="https://docs.wiris.com/mathtype/en/">Limit Variations</a>):</td></tr><tr><td><strong>selector</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td><td><strong>class</strong></td></tr><tr><td>23</td><td>tmLIM</td><td>limits</td><td><a href="https://docs.wiris.com/mathtype/en/">LimBoxClass</a></td></tr><tr><td><strong>variations</strong></td><td><strong>variation</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td></tr><tr><td>:::</td><td>0</td><td>tvSUBAR</td><td>single underbar</td></tr><tr><td>:::</td><td>1</td><td>tvDUBAR</td><td>double underbar</td></tr><tr><td colspan="4">Horizontal braces and brackets:</td></tr><tr><td><strong>selector</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td><td><strong>class</strong></td></tr><tr><td>24</td><td>tmHBRACE</td><td>horizontal brace</td><td><a href="https://docs.wiris.com/mathtype/en/">HFenceBoxClass</a></td></tr><tr><td>25</td><td>tmHBRACK</td><td>horizontal bracket</td><td><a href="https://docs.wiris.com/mathtype/en/">HFenceBoxClass</a></td></tr><tr><td><strong>variations</strong></td><td><strong>variation</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td></tr><tr><td>:::</td><td>0×0001</td><td>tvHB_TOP</td><td>slot is on the top, else on the bottom</td></tr><tr><td colspan="4">Long division:</td></tr><tr><td><strong>selector</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td><td><strong>class</strong></td></tr><tr><td>26</td><td>tmLDIV</td><td>long division</td><td><a href="https://docs.wiris.com/mathtype/en/">LDivBoxClass</a></td></tr><tr><td><strong>variations</strong></td><td><strong>variation</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td></tr><tr><td>:::</td><td>0×0001</td><td>tvLD_UPPER</td><td>upper slot is present</td></tr><tr><td colspan="4">Subscripts and superscripts:</td></tr><tr><td><strong>selector</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td><td><strong>class</strong></td></tr><tr><td>27</td><td>tmSUB</td><td>subscript</td><td><a href="https://docs.wiris.com/mathtype/en/">ScrBoxClass</a></td></tr><tr><td>28</td><td>tmSUP</td><td>superscript</td><td><a href="https://docs.wiris.com/mathtype/en/">ScrBoxClass</a></td></tr><tr><td>29</td><td>tmSUBSUP</td><td>subscript and superscript</td><td><a href="https://docs.wiris.com/mathtype/en/">ScrBoxClass</a></td></tr><tr><td><strong>variations</strong></td><td><strong>variation</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td></tr><tr><td>:::</td><td>0×0001</td><td>tvSU_PRECEDES</td><td>script precedes scripted item, else follows</td></tr><tr><td colspan="4">Dirac bra-ket notation:</td></tr><tr><td><strong>selector</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td><td><strong>class</strong></td></tr><tr><td>30</td><td>tmDIRAC</td><td>bra-ket notation</td><td><a href="https://docs.wiris.com/mathtype/en/">DiracBoxClass</a></td></tr><tr><td><strong>variations</strong></td><td><strong>variation</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td></tr><tr><td>:::</td><td>0×0001</td><td>tvDI_LEFT</td><td>left part is present</td></tr><tr><td>:::</td><td>0×0002</td><td>tvDI_RIGHT</td><td>right part is present</td></tr><tr><td colspan="4">Vectors:</wrap></td></tr><tr><td><strong>selector</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td><td><strong>class</strong></td></tr><tr><td>31</td><td>tmVEC</td><td>vector</td><td>HatBoxClass</td></tr><tr><td><strong>variations</strong></td><td><strong>variation</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td></tr><tr><td>:::</td><td>0×0001</td><td>tvVE_LEFT</td><td>arrow points left</td></tr><tr><td>:::</td><td>0×0002</td><td>tvVE_RIGHT</td><td>arrow points right</td></tr><tr><td>:::</td><td>0×0004</td><td>tvVE_UNDER</td><td>arrow under slot, else over slot</td></tr><tr><td>:::</td><td>0×0008</td><td>tvVE_HARPOON</td><td>harpoon</td></tr><tr><td colspan="4">Hats, arcs, tilde, joint status:</td></tr><tr><td><strong>selector</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td><td><strong>class</strong></td></tr><tr><td>32</td><td>tmTILDE</td><td>tilde over characters</td><td>HatBoxClass</td></tr><tr><td>33</td><td>tmHAT</td><td>hat over characters</td><td>HatBoxClass</td></tr><tr><td>34</td><td>tmARC</td><td>arc over characters</td><td>HatBoxClass</td></tr><tr><td>35</td><td>tmJSTATUS</td><td>joint status construct</td><td>HatBoxClass</td></tr><tr><td colspan="4">Overstrikes (cross-outs):</td></tr><tr><td><strong>selector</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td><td><strong>class</strong></td></tr><tr><td>36</td><td>tmSTRIKE</td><td>overstrike (cross-out)</td><td>StrikeBoxClass</td></tr><tr><td><strong>variations</strong></td><td><strong>variation</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td></tr><tr><td>:::</td><td>0×0001</td><td>tvST_HORIZ</td><td>line is horizontal, else slashes</td></tr><tr><td>:::</td><td>0×0002</td><td>tvST_UP</td><td>if slashes, slash from lower-left to upper-right is present</td></tr><tr><td>:::</td><td>0×0004</td><td>tvST_DOWN</td><td>if slashes, slash from upper-left to lower-right is present</td></tr><tr><td colspan="4">Boxes:</wrap></td></tr><tr><td><strong>selector</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td><td><strong>class</strong></td></tr><tr><td>37</td><td>tmBOX</td><td>box</td><td>TBoxBoxClass</td></tr><tr><td><strong>variations</strong></td><td><strong>variation</strong></td><td><strong>symbol</strong></td><td><strong>description</strong></td></tr><tr><td>:::</td><td>0×0001</td><td>tvBX_ROUND</td><td>corners are round, else square</td></tr><tr><td>:::</td><td>0×0002</td><td>tvBX_LEFT</td><td>left side is present</td></tr><tr><td>:::</td><td>0×0004</td><td>tvBX_RIGHT</td><td>right side is present</td></tr><tr><td>:::</td><td>0×0008</td><td>tvBX_TOP</td><td>top side is present</td></tr><tr><td>:::</td><td>0×0010</td><td>tvBX_BOTTOM</td><td>bottom side is present</td></tr></tbody></table>

### Template subobject order:

Template subobjects appear in object lists in the order that matches the movement of the insertion point within a template in the MathType user interface. For template classes that have more than one slot, the following list shows the order of subobjects for all templates with that class:

ArroBoxClass: expanding arrows

- main slot
- arrow character

BigOpBoxClass: integrals, summations, products, etc.

- main slot (summand, integrand)
- upper slot
- lower slot
- large operator character

DiracBoxClass: three-part bracket structure

- left slot
- right slot
- left angle bracket (optional)
- vertical bar
- right angle bracket (optional)

FracBoxClass: fractions with a horizontal bar

- numerator slot
- denominator slot

HFenceBoxClass horizontal expanding braces

- main slot
- small slot
- brace character

LDivBoxClass long division

- dividend slot
- quotient slot

LimBoxClass limits

- main slot
- lower slot
- upper slot

ParBoxClass parenthesized or bracketed slots

- main slot
- left fence character (optional)
- right fence character (optional)

RootBoxClass radical sign

- main slot
- radicand slot

ScrBoxClass subscripts and superscripts

- subscript slot
- superscript slot

SlashBoxClass fractions with a diagonal bar

- numerator slot
- denominator slot

## Example

In order to illustrate with a concrete example, in this section will examine the byte stream in detail for the quadratic formula:

![quadratic2.gif](https://static.helpjuice.com/helpjuice_production/uploads/upload/image/23810/direct/1730376936790/uuid-ee92ed47-084d-7c4d-c045-8e5150179f62.gif)

For this equation, MathType 7.0 for Windows generates the following:

| byte position | byte value | meaning | record |  |
| --- | --- | --- | --- | --- |
| 0 | 0×0000 | 5, 0×05 | version | [header](https://docs.wiris.com/mathtype/en/) |
| 1 | 0×0001 | 1, 0×01 | Windows | ::: |
| 2 | 0×0002 | 0, 0×00 | MathType | ::: |
| 3 | 0×0003 | 7, 0×07 | MT major version | ::: |
| 4 | 0×0004 | 0, 0×00 | MT minor version | ::: |
| 5 | 0×0005 | 68, 0×44, 'D' | application key | ::: |
| 6 | 0×0006 | 83, 0×53, 'S' | ::: | ::: |
| 7 | 0×0007 | 77, 0×4D, 'M' | ::: | ::: |
| 8 | 0×0008 | 84, 0×54, 'T' | ::: | ::: |
| 9 | 0×0009 | 55, 0×37, '7' | ::: | ::: |
| 10 | 0×000A | 0, 0×00 | ::: | ::: |
| 11 | 0×000B | 0, 0×00 | equation options | ::: |
| 12 | 0×000C | 19, 0×13 | record type | [ENCODING\_DEF](https://docs.wiris.com/mathtype/en/) encoding #5 |
| 13 | 0×000D | 87, 0×57, 'W' | encoding name "WinAllBasicCodePages" | ::: |
| 14 | 0×000E | 105, 0×69, 'i' | ::: | ::: |
| 15 | 0×000F | 110, 0×6E, 'n' | ::: | ::: |
| 16 | 0×0010 | 65, 0×41, 'A' | ::: | ::: |
| 17 | 0×0011 | 108, 0×6C, 'l' | ::: | ::: |
| 18 | 0×0012 | 108, 0×6C, 'l' | ::: | ::: |
| 19 | 0×0013 | 66, 0×42, 'B' | ::: | ::: |
| 20 | 0×0014 | 97, 0×61, 'a' | ::: | ::: |
| 21 | 0×0015 | 115, 0×73, 's' | ::: | ::: |
| 22 | 0×0016 | 105, 0×69, 'i' | ::: | ::: |
| 23 | 0×0017 | 99, 0×63, 'c' | ::: | ::: |
| 24 | 0×0018 | 67, 0×43, 'C' | ::: | ::: |
| 25 | 0×0019 | 111, 0×6F, 'o' | ::: | ::: |
| 26 | 0×001A | 100, 0×64, 'd' | ::: | ::: |
| 27 | 0×001B | 101, 0×65, 'e' | ::: | ::: |
| 28 | 0×001C | 80, 0×50, 'P' | ::: | ::: |
| 29 | 0×001D | 97, 0×61, 'a' | ::: | ::: |
| 30 | 0×001E | 103, 0×67, 'g' | ::: | ::: |
| 31 | 0×001F | 101, 0×65, 'e' | ::: | ::: |
| 32 | 0×0020 | 115, 0×73, 's' | ::: | ::: |
| 33 | 0×0021 | 0, 0×00 | ::: | ::: |
| 34 | 0×0022 | 17, 0×11 | record type | [FONT\_DEF](https://docs.wiris.com/mathtype/en/) font def #1 |
| 35 | 0×0023 | 5, 0×05 | index of encoding WinAllBasicCodePages | ::: |
| 36 | 0×0024 | 84, 0×54, 'T' | font name "Times New Roman" | ::: |
| 37 | 0×0025 | 105, 0×69, 'i' | ::: | ::: |
| 38 | 0×0026 | 109, 0×6D, 'm' | ::: | ::: |
| 39 | 0×0027 | 101, 0×65, 'e' | ::: | ::: |
| 40 | 0×0028 | 115, 0×73, 's' | ::: | ::: |
| 41 | 0×0029 | 32, 0×20, ' ' | ::: | ::: |
| 42 | 0×002A | 78, 0×4E, 'N' | ::: | ::: |
| 43 | 0×002B | 101, 0×65, 'e' | ::: | ::: |
| 44 | 0×002C | 119, 0×77, 'w' | ::: | ::: |
| 45 | 0×002D | 32, 0×20, ' ' | ::: | ::: |
| 46 | 0×002E | 82, 0×52, 'R' | ::: | ::: |
| 47 | 0×002F | 111, 0×6F, 'o' | ::: | ::: |
| 48 | 0×0030 | 109, 0×6D, 'm' | ::: | ::: |
| 49 | 0×0031 | 97, 0×61, 'a' | ::: | ::: |
| 50 | 0×0032 | 110, 0×6E, 'n' | ::: | ::: |
| 51 | 0×0033 | 0, 0×00 | ::: | ::: |
| 52 | 0×0034 | 17, 0×11 | record type | [FONT\_DEF](https://docs.wiris.com/mathtype/en/) font def #2 |
| 53 | 0×0035 | 3, 0×03 | index of encoding Symbol | ::: |
| 54 | 0×0036 | 83, 0×53, 'S' | font name "Symbol" | ::: |
| 55 | 0×0037 | 121, 0×79, 'y' | ::: | ::: |
| 56 | 0×0038 | 109, 0×6D, 'm' | ::: | ::: |
| 57 | 0×0039 | 98, 0×62, 'b' | ::: | ::: |
| 58 | 0×003A | 111, 0×6F, 'o' | ::: | ::: |
| 59 | 0×003B | 108, 0×6C, 'l' | ::: | ::: |
| 60 | 0×003C | 0, 0×00 | ::: | ::: |
| 61 | 0×003D | 17, 0×11 | record type | [FONT\_DEF](https://docs.wiris.com/mathtype/en/) font def #3 |
| 62 | 0×003E | 5, 0×05 | index of encoding WinAllBasicCodePages | ::: |
| 63 | 0×003F | 67, 0×43, 'C' | font name "Courier New" | ::: |
| 64 | 0×0040 | 111, 0×6F, 'o' | ::: | ::: |
| 65 | 0×0041 | 117, 0×75, 'u' | ::: | ::: |
| 66 | 0×0042 | 114, 0×72, 'r' | ::: | ::: |
| 67 | 0×0043 | 105, 0×69, 'i' | ::: | ::: |
| 68 | 0×0044 | 101, 0×65, 'e' | ::: | ::: |
| 69 | 0×0045 | 114, 0×72, 'r' | ::: | ::: |
| 70 | 0×0046 | 32, 0×20, ' ' | ::: | ::: |
| 71 | 0×0047 | 78, 0×4E, 'N' | ::: | ::: |
| 72 | 0×0048 | 101, 0×65, 'e' | ::: | ::: |
| 73 | 0×0049 | 119, 0×77, 'w' | ::: | ::: |
| 74 | 0×004A | 0, 0×00 | ::: | ::: |
| 75 | 0×004B | 17, 0×11 | record type | [FONT\_DEF](https://docs.wiris.com/mathtype/en/) font def #4 |
| 76 | 0×004C | 4, 0×04 | index of encoding MTExtra | ::: |
| 77 | 0×004D | 77, 0×4D, 'M' | font name "MT Extra" | ::: |
| 78 | 0×004E | 84, 0×54, 'T' | ::: | ::: |
| 79 | 0×004F | 32, 0×20, ' ' | ::: | ::: |
| 80 | 0×0050 | 69, 0×45, 'E' | ::: | ::: |
| 81 | 0×0051 | 120, 0×78, 'x' | ::: | ::: |
| 82 | 0×0052 | 116, 0×74, 't' | ::: | ::: |
| 83 | 0×0053 | 114, 0×72, 'r' | ::: | ::: |
| 84 | 0×0054 | 97, 0×61, 'a' | ::: | ::: |
| 85 | 0×0055 | 0, 0×00 | ::: | ::: |
| 86 | 0×0056 | 18, 0×12 | record type | [EQN\_PREFS](https://docs.wiris.com/mathtype/en/) |
| 87 | 0×0057 | 0, 0×00 | options | ::: |
| 88 | 0×0058 | 8, 0×08 | size array count | ::: |
| 89 | 0×0059 | 33, 0×21, '!' | nibble array containing: size #1: "12 points" size #2: "58 %" size #3: "42 %" size #4: "150 %" size #5: "100 %" size #6: "75 %" size #7: "150 %" size #8: "1 point" | ::: |
| 90 | 0×005A | 47, 0×2F, '/' | ::: | ::: |
| 91 | 0×005B | 69, 0×45, 'E' | ::: | ::: |
| 92 | 0×005C | 143, 0×8F | ::: | ::: |
| 93 | 0×005D | 68, 0×44, 'D' | ::: | ::: |
| 94 | 0×005E | 47, 0×2F, '/' | ::: | ::: |
| 95 | 0×005F | 65, 0×41, 'A' | ::: | ::: |
| 96 | 0×0060 | 80, 0×50, 'P' | ::: | ::: |
| 97 | 0×0061 | 244, 0×F4 | ::: | ::: |
| 98 | 0×0062 | 16, 0×10 | ::: | ::: |
| 99 | 0×0063 | 15, 0×0F | ::: | ::: |
| 100 | 0×0064 | 71, 0×47, 'G' | ::: | ::: |
| 101 | 0×0065 | 95, 0×5F, '\_' | ::: | ::: |
| 102 | 0×0066 | 65, 0×41, 'A' | ::: | ::: |
| 103 | 0×0067 | 80, 0×50, 'P' | ::: | ::: |
| 104 | 0×0068 | 242, 0×F2 | ::: | ::: |
| 105 | 0×0069 | 31, 0×1F | ::: | ::: |
| 106 | 0×006A | 30, 0×1E | spacing array count | ::: |
| 107 | 0×006B | 65, 0×41, 'A' | nibble array containing 30 values | ::: |
| 108 | 0×006C | 80, 0×50, 'P' | ::: | ::: |
| 109 | 0×006D | 244, 0×F4 | ::: | ::: |
| 110 | 0×006E | 21, 0×15 | ::: | ::: |
| 111 | 0×006F | 15, 0×0F | ::: | ::: |
| 112 | 0×0070 | 65, 0×41, 'A' | ::: | ::: |
| 113 | 0×0071 | 0, 0×00 | ::: | ::: |
| 114 | 0×0072 | 244, 0×F4 | ::: | ::: |
| 115 | 0×0073 | 69, 0×45, 'E' | ::: | ::: |
| 116 | 0×0074 | 244, 0×F4 | ::: | ::: |
| 117 | 0×0075 | 37, 0×25, '%' | ::: | ::: |
| 118 | 0×0076 | 244, 0×F4 | ::: | ::: |
| 119 | 0×0077 | 143, 0×8F | ::: | ::: |
| 120 | 0×0078 | 66, 0×42, 'B' | ::: | ::: |
| 121 | 0×0079 | 95, 0×5F, '\_' | ::: | ::: |
| 122 | 0×007A | 65, 0×41, 'A' | ::: | ::: |
| 123 | 0×007B | 0, 0×00 | ::: | ::: |
| 124 | 0×007C | 244, 0×F4 | ::: | ::: |
| 125 | 0×007D | 16, 0×10 | ::: | ::: |
| 126 | 0×007E | 15, 0×0F | ::: | ::: |
| 127 | 0×007F | 67, 0×43, 'C' | ::: | ::: |
| 128 | 0×0080 | 95, 0×5F, '\_' | ::: | ::: |
| 129 | 0×0081 | 65, 0×41, 'A' | ::: | ::: |
| 130 | 0×0082 | 0, 0×00 | ::: | ::: |
| 131 | 0×0083 | 242, 0×F2 | ::: | ::: |
| 132 | 0×0084 | 31, 0×1F | ::: | ::: |
| 133 | 0×0085 | This error message can appear when you open Word or you click in the remnant 'MathType' tab in Word: | ::: | ::: |
| 134 | 0×0086 | 165, 0×A5 | ::: | ::: |
| 135 | 0×0087 | 242, 0×F2 | ::: | ::: |
| 136 | 0×0088 | 10, 0×0A | ::: | ::: |
| 137 | 0×0089 | 37, 0×25 | ::: | ::: |
| 138 | 0×008A | 244, 0×F4 | ::: | ::: |
| 139 | 0×008B | 143, 0×8F | ::: | ::: |
| 140 | 0×008C | 65, 0×41, '!' | ::: | ::: |
| 141 | 0×008D | 244, 0×F4 | ::: | ::: |
| 142 | 0×008E | 16, 0×10 | ::: | ::: |
| 143 | 0×008F | 15, 0×0F | ::: | ::: |
| 144 | 0×0090 | 64, 0×41, 'A' | ::: | ::: |
| 145 | 0×0091 | 0, 0×00 | ::: | ::: |
| 146 | 0×0092 | 244, 0×F4 | ::: | ::: |
| 147 | 0×0093 | 15, 0×0F | ::: | ::: |
| 148 | 0×0094 | 72, 0×48, 'H' | ::: | ::: |
| 149 | 0×0095 | 244, 0×F4 | ::: | ::: |
| 150 | 0×0096 | 23, 0×17 | ::: | ::: |
| 151 | 0×0097 | 244, 0×F4 | ::: | ::: |
| 152 | 0×0098 | 143, 0×8F | ::: | ::: |
| 153 | 0×0099 | 64, 0×41, 'A' | ::: | ::: |
| 154 | 0×009A | 0, 0×00 | ::: | ::: |
| 155 | 0×009B | 242, 0×F2 | ::: | ::: |
| 156 | 0×009C | 26, 0×1A | ::: | ::: |
| 157 | 0×009D | 95, 0×5F, '\_' | ::: | ::: |
| 158 | 0×009E | 68, 0×44, 'D' | ::: | ::: |
| 159 | 0×009F | 95, 0×5F, '\_' | ::: | ::: |
| 160 | 0×00A0 | 69, 0×45, 'E' | ::: | ::: |
| 161 | 0×00A1 | 244, 0×F4 | ::: | ::: |
| 162 | 0×00A2 | 95, 0×5F, '\_' | ::: | ::: |
| 163 | 0×00A3 | 69, 0×45, 'E' | ::: | ::: |
| 164 | 0×00A4 | 244, 0×F4 | ::: | ::: |
| 165 | 0×00A5 | 95, 0×5F, '\_' | ::: | ::: |
| 166 | 0×00A6 | 65, 0×41, 'A' | ::: | ::: |
| 167 | 0×00A7 | 15, 0×0F | ::: | ::: |
| 168 | 0×00A8 | 12, 0×0C | style array count | ::: |
| 169 | 0×00A9 | 1, 0×01 | style #1: font def #1, plain | ::: |
| 170 | 0×00AA | 0, 0×00 | ::: | ::: |
| 171 | 0×00AB | 1, 0×01 | style #2: font def #1, plain | ::: |
| 172 | 0×00AC | 0, 0×00 | ::: | ::: |
| 173 | 0×00AD | 1, 0×01 | style #3: font def #1, italic | ::: |
| 174 | 0×00AE | 2, 0×02 | ::: | ::: |
| 175 | 0×00AF | 2, 0×02 | style #4: font def #2, italic | ::: |
| 176 | 0×00B0 | 2, 0×02 | ::: | ::: |
| 177 | 0×00B1 | 2, 0×02 | style #5: font def #2, plain | ::: |
| 178 | 0×00B2 | 0, 0×00 | ::: | ::: |
| 179 | 0×00B3 | 2, 0×02 | style #6: font def #2, plain | ::: |
| 180 | 0×00B4 | 0, 0×00 | ::: | ::: |
| 181 | 0×00B5 | 1, 0×01 | style #7: font def #1, bold | ::: |
| 182 | 0×00B6 | 1, 0×01 | ::: | ::: |
| 183 | 0×00B7 | 1, 0×01 | style #8: font def #1, plain | ::: |
| 184 | 0×00B8 | 0, 0×00 | ::: | ::: |
| 185 | 0×00B9 | 3, 0×03 | style #9: font def #3, plain | ::: |
| 186 | 0×00BA | 0, 0×00 | ::: | ::: |
| 187 | 0×00BB | 1, 0×01 | style #10: font def #1, plain | ::: |
| 188 | 0×00BC | 0, 0×00 | ::: | ::: |
| 189 | 0×00BD | 4, 0×04 | style #11: font def #4, plain | ::: |
| 190 | 0×00BE | 0, 0×00 | ::: | ::: |
| 191 | 0×00BF | 0, 0×00 | style #12: (not used) | ::: |
| 192 | 0×00C0 | 10, 0×0A | record type | [SIZE\_FULL](https://docs.wiris.com/mathtype/en/) |
| 193 | 0×00C1 | 1, 0×01 | record type | [LINE](https://docs.wiris.com/mathtype/en/) |
| 194 | 0×00C2 | 0, 0×00 | options | ::: |
| 195 | 0×00C3 | 3, 0×03 | record type | [TMPL](https://docs.wiris.com/mathtype/en/) fraction |
| 196 | 0×00C4 | 0, 0×00 | options | ::: |
| 197 | 0×00C5 | 11, 0×0B | selector: tmFRACT | ::: |
| 198 | 0×00C6 | 0, 0×00 | variation: none | ::: |
| 199 | 0×00C7 | 0, 0×00 | template-specific options | ::: |
| 200 | 0×00C8 | 1, 0×01 | record type | [LINE](https://docs.wiris.com/mathtype/en/) numerator |
| 201 | 0×00C9 | 0, 0×00 | options | ::: |
| 202 | 0×00CA | 2, 0×02 | record type | [CHAR](https://docs.wiris.com/mathtype/en/) – (minus sign) |
| 203 | 0×00CB | 4, 0×04 | options: [OPT\_CHAR\_ENC\_CHAR\_8](https://docs.wiris.com/mathtype/en/) | ::: |
| 204 | 0×00CC | 134, 0×86 | typeface: 134 - 128 = 6 (Symbol style) | ::: |
| 205 | 0×00CD | 18, 0×12 | MTCode value: 0×2212 (minus sign) | ::: |
| 206 | 0×00CE | 34, 0×22, '""""' | ::: | ::: |
| 207 | 0×00CF | 45, 0×2D, '-' | font-encoded value: 0×2D (minus sign) | ::: |
| 208 | 0×00D0 | 2, 0×02 | record type | [CHAR](https://docs.wiris.com/mathtype/en/) b |
| 209 | 0×00D1 | 0, 0×00 | options | ::: |
| 210 | 0×00D2 | 131, 0×83 | typeface: 131 - 128 = 3 (Variable style) | ::: |
| 211 | 0×00D3 | 98, 0×62, 'b' | MTCode value: 0×0062 ('b') | ::: |
| 212 | 0×00D4 | 0, 0×00 | ::: | ::: |
| 213 | 0×00D5 | 2, 0×02 | record type | [CHAR](https://docs.wiris.com/mathtype/en/) plus-minus sign: ± |
| 214 | 0×00D6 | 4, 0×04 | options: [OPT\_CHAR\_ENC\_CHAR\_8](https://docs.wiris.com/mathtype/en/) | ::: |
| 215 | 0×00D7 | 134, 0×86 | typeface: 134 - 128 = 6 (Symbol style) | ::: |
| 216 | 0×00D8 | 177, 0×B1 | MTCode value: 0×00B1 (plus-minus sign) | ::: |
| 217 | 0×00D9 | 0, 0×00 | ::: | ::: |
| 218 | 0×00DA | 177, 0×B1 | font-encoded value: 0×B1 (plus-minus sign) | ::: |
| 219 | 0×00DB | 3, 0×03 | record type | [TMPL](https://docs.wiris.com/mathtype/en/) square root |
| 220 | 0×00DC | 0, 0×00 | options | ::: |
| 221 | 0×00DD | 10, 0×0A | selector: tmROOT | ::: |
| 222 | 0×00DE | 0, 0×00 | variations: tvROOT\_SQ (square root) | ::: |
| 223 | 0×00DF | 0, 0×00 | template-specific options | ::: |
| 224 | 0×00E0 | 1, 0×01 | record type | [LINE](https://docs.wiris.com/mathtype/en/) radicand |
| 225 | 0×00E1 | 0, 0×00 | options | ::: |
| 226 | 0×00E2 | 2, 0×02 | record type | [CHAR](https://docs.wiris.com/mathtype/en/) b |
| 227 | 0×00E3 | 0, 0×00 | options | ::: |
| 228 | 0×00E4 | 131, 0×83 | typeface: 131 - 128 = 3 (Variable style) | ::: |
| 229 | 0×00E5 | 98, 0×62, 'b' | MTCode value: 0×0062 ('b') | ::: |
| 230 | 0×00E6 | 0, 0×00 | ::: | ::: |
| 231 | 0×00E7 | 3, 0×03 | record type | [TMPL](https://docs.wiris.com/mathtype/en/) superscript |
| 232 | 0×00E8 | 0, 0×00 | options | ::: |
| 233 | 0×00E9 | 28, 0×1C | selector: tmSUP | ::: |
| 234 | 0×00EA | 0, 0×00 | variations: none | ::: |
| 235 | 0×00EB | 0, 0×00 | template-specific options | ::: |
| 236 | 0×00EC | 11, 0×0B | record type | [SIZE\_SUB](https://docs.wiris.com/mathtype/en/) |
| 237 | 0×00ED | 1, 0×01 | record type | [LINE](https://docs.wiris.com/mathtype/en/) missing superscript |
| 238 | 0×00EE | 1, 0×01 | options: [OPT\_LINE\_NULL](https://docs.wiris.com/mathtype/en/) | ::: |
| 239 | 0×00EF | 1, 0×01 | record type | [LINE](https://docs.wiris.com/mathtype/en/) subscript |
| 240 | 0×00F0 | 0, 0×00 | options | ::: |
| 241 | 0×00F1 | 2, 0×02 | record type | [CHAR](https://docs.wiris.com/mathtype/en/) 2 |
| 242 | 0×00F2 | 0, 0×00 | options | ::: |
| 243 | 0×00F3 | 136, 0×88 | typeface: 136 - 128 = 8 (Number style) | ::: |
| 244 | 0×00F4 | 50, 0×32, '2' | MTCode value: 0×0032 ('2') | ::: |
| 245 | 0×00F5 | 0, 0×00 | ::: | ::: |
| 246 | 0×00F6 | 0, 0×00 | record type | [END](https://docs.wiris.com/mathtype/en/) of subscript line |
| 247 | 0×00F7 | 0, 0×00 | record type | [END](https://docs.wiris.com/mathtype/en/) of subscript template |
| 248 | 0×00F8 | 10, 0×0A | record type | [SIZE\_FULL](https://docs.wiris.com/mathtype/en/) |
| 249 | 0×00F9 | 2, 0×02 | record type | [CHAR](https://docs.wiris.com/mathtype/en/) – (minus sign) |
| 250 | 0×00FA | 4, 0×04 | options: [OPT\_CHAR\_ENC\_CHAR\_8](https://docs.wiris.com/mathtype/en/) | ::: |
| 251 | 0×00FB | 134, 0×86 | typeface: 134 - 128 = 6 (Symbol style) | ::: |
| 252 | 0×00FC | 18, 0×12 | MTCode value: 0×2212 (minus sign) | ::: |
| 253 | 0×00FD | 34, 0×22, '""""' | ::: | ::: |
| 254 | 0×00FE | 45, 0×2D, '-' | font-encoded value: 0×2D (minus sign) | ::: |
| 255 | 0×00FF | 38, 0×26 | ::: | ::: |
| 256 | 0×0100 | 2, 0×02 | record type | [CHAR](https://docs.wiris.com/mathtype/en/) 4 |
| 257 | 0×0101 | 0, 0×00 | options | ::: |
| 258 | 0×0102 | 136, 0×88 | typeface: 136 - 128 = 8 (Number style) | ::: |
| 259 | 0×0103 | 52, 0×34, '4' | MTCode value: 0×0034 ('4') | ::: |
| 260 | 0×0104 | 0, 0×00 | ::: | ::: |
| 261 | 0×0105 | 2, 0×02 | record type | [CHAR](https://docs.wiris.com/mathtype/en/) a |
| 262 | 0×0106 | 0, 0×00 | options | ::: |
| 263 | 0×0107 | 131, 0×83 | typeface: 131 - 128 = 3 (Variable style) | ::: |
| 264 | 0×0108 | 97, 0×61, 'a' | MTCode value: 0×0061 ('a') | ::: |
| 265 | 0×0109 | 0, 0×00 | ::: | ::: |
| 266 | 0×010A | 2, 0×02 | record type | [CHAR](https://docs.wiris.com/mathtype/en/) c |
| 267 | 0×010B | 0, 0×00 | options | ::: |
| 268 | 0×010C | 131, 0×83 | typeface: 131 - 128 = 3 (Variable style) | ::: |
| 269 | 0×010D | 99, 0×63, 'c' | MTCode value: 0×0063 ('c') | ::: |
| 270 | 0×010E | 0, 0×00 | ::: | ::: |
| 271 | 0×010F | 0, 0×00 | record type | [END](https://docs.wiris.com/mathtype/en/) of radicand line |
| 272 | 0×0110 | 11, 0×0B | record type | [SIZE\_SUB](https://docs.wiris.com/mathtype/en/) |
| 273 | 0×0111 | 1, 0×01 | record type | [LINE](https://docs.wiris.com/mathtype/en/) missing radicand |
| 274 | 0×0112 | 1, 0×01 | options: [OPT\_LINE\_NULL](https://docs.wiris.com/mathtype/en/) | ::: |
| 275 | 0×0113 | 0, 0×00 | record type | [END](https://docs.wiris.com/mathtype/en/) of radical template |
| 276 | 0×0114 | 0, 0×00 | record type | [END](https://docs.wiris.com/mathtype/en/) of numerator line |
| 277 | 0×0115 | 10, 0×0A | record type | [SIZE\_FULL](https://docs.wiris.com/mathtype/en/) |
| 278 | 0×0116 | 1, 0×01 | record type | [LINE](https://docs.wiris.com/mathtype/en/) denominator |
| 279 | 0×0117 | 0, 0×00 | options | ::: |
| 280 | 0×0118 | 2, 0×02 | record type | [CHAR](https://docs.wiris.com/mathtype/en/) 2 |
| 281 | 0×0119 | 0, 0×00 | options | ::: |
| 282 | 0×011A | 136, 0×88 | typeface: 136 - 128 = 8 (Number style) | ::: |
| 283 | 0×011B | 50, 0×32, '2' | MTCode value: 0×0032 ('2') | ::: |
| 284 | 0×011C | 0, 0×00 | ::: | ::: |
| 285 | 0×011D | 2, 0×02 | record type | [CHAR](https://docs.wiris.com/mathtype/en/) a |
| 286 | 0×011E | 0, 0×00 | options | ::: |
| 287 | 0×011F | 131, 0×83 | typeface: 131 - 128 = 3 (Variable style) | ::: |
| 288 | 0×0120 | 97, 0×61, 'a' | MTCode value: 0×0061 ('a') | ::: |
| 289 | 0×0121 | 0, 0×00 | ::: | ::: |
| 290 | 0×0122 | 0, 0×00 | record type | [END](https://docs.wiris.com/mathtype/en/) of denominator line |
| 291 | 0×0123 | 0, 0×00 | record type | [END](https://docs.wiris.com/mathtype/en/) of fraction template |
| 292 | 0×0124 | 0, 0×00 | record type | [END](https://docs.wiris.com/mathtype/en/) of equation line |
| 293 | 0×0125 | 0, 0×00 | record type | [END](https://docs.wiris.com/mathtype/en/) of equation |