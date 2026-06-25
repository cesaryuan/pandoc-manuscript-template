---
title: "How MTEF is stored in files and objects"
source: "https://docs.wiris.com/en_US/mathtype-sdk-documentation/how-mtef-is-stored-in-files-and-objects"
author:
published:
created: 2026-06-25
description: "Abstract This document describes how MTEF, the binary equation format used by MathType is embedded in OLE equation objects produced by MathType as well as"
tags:
  - "clippings"
---
- ## MathType
	- ## Getting started
		- ## Using MathType
		- ## Technical documentation
		- ## MathType 7
			- ## MathType SDK
				- ## Getting started to MathType's API
								- - ## Application-specific metafile comment convention
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

This document describes how MTEF, the binary equation format used by MathType is embedded in OLE equation objects produced by MathType as well as in all the file formats in which MathType can save equations. The MTEF format itself is described in other documents (see [MTEF 4](https://docs.wiris.com/mathtype/en/mathtype-office-tools/mathtype-7-for-windows-and-mac/mathtype-sdk/MTEF4) and [MTEF 5](https://docs.wiris.com/mathtype/en/mathtype-office-tools/mathtype-7-for-windows-and-mac/mathtype-sdk/MTEF5)).

## Introduction

MathType can save its equations in a variety of file formats and object types. So that MathType can re-open such equations, it must store its own equation data structures (MTEF) in each file. Most of these file types are standards that are not under Wiris's control and, therefore, there is no opportunity to make MTEF an official part of the file format. Luckily, the designers of these file formats have seen the need to store application-specific data and have provided mechanisms to allow this. The rest of this document describes how MathType makes use of such mechanisms for storing MTEF.

## Mac Picture (PICT)

The Mac PICT format is the Mac's native graphics metafile (picture) format and is used in PICT files, in OLE objects, and on the clipboard. In the PICT format, MTEF is stored as a application-specific picture comment (see [Mac Technical Note QD06](http://developer.apple.com/legacy/mac/library/technotes/qd/qd_06.html)), where the contents consists of a header followed by the MTEF data:

```
// picture comment header
typedef struct {
 long appl_sig; // 'MATH'
 short local_kind; // 1 for len and checksum present, 0 if not
 short len; // length of data in bytes
 short checksum;
 // followed by the MTEF data
} PComHeader;
```

## Windows MetaFile (WMF)

The Windows Metafile format is Microsoft Window's native graphics metafile (picture) format and is used in WMF files, on the clipboard, and in OLE objects.

MTEF is embedded in WMF data using the MFCOMMENT escape function. MathType 6 and later use a different format for this comment than previous versions of MathType.

### MathType WMF comment format

For MathType 6.0b and later versions, the format of the MTEF data embedded in WMF is described in [Application-specific Metafile Comment Convention](https://docs.wiris.com/mathtype/en/mathtype-office-tools/mathtype-7-for-windows-and-mac/mathtype-sdk/AppSpecificMFCmtConv).

The main reason for the different format is to get around the 32Kb limit on the size of data allowed in a single comment. MathType uses this same format for storing MathML as well. See [How MathML is Stored in Files and Objects](https://docs.wiris.com/mathtype/en/mathtype-office-tools/mathtype-7-for-windows-and-mac/mathtype-sdk/MathMLstorage) for more details.

### Pre-MathType 6 WMF comment format

For versions of MathType prior to MathType 6.0b, the escape data consists of a 12-byte header followed by the MTEF data:

```
// MTEF escape header
#pragma pack(pop)
typedef struct {
 char mathtype[8]; // "MathType"
 short magic; // 0x5555
 short len; // length of data in bytes
 // followed by the MTEF data
} MTEFEscHeader;
#pragma pack(pop)
```

The total length of the header and MTEF data must be less than 32Kb. Programmers should note that the #pragma pack statement shown above are Microsoft compiler control statements that force the struct to be byte aligned (so it is exactly 12 bytes).

## Encapsulated PostScript (EPS)

MTEF data is stored in an EPS file as a PostScript comment immediately following the header required by the EPS format and preceding the MathML (for MathType version 6 or later, for Windows only, See: [How MathML is Stored in EPS](https://docs.wiris.com/mathtype/en/mathtype-office-tools/mathtype-7-for-windows-and-mac/mathtype-sdk/MathMLstorage#Encapsulated%20PostScript%20\(EPS\))) and the PostScript code generated by MathType to draw the equation. The first line identifies the comment and the following lines contain the MTEF data encoded into readable ASCII, followed by a checksum. For example, this is the MTEF for the expression *x* + *y*:

```
%MathType!MTEF!1!1!+-
%feaahaart1ev3aaatCvAUfeBSjuyZL2yd9gzLbvyNv2CaerbuL
%wBLnhiov2DGi1BTfMBaeXatLxBI9gBaerbd9wDYLwzYbItLDha
%rqqtubsr4rNCHbGeaGqiVu0Je9sqqrpepC0xbbL8F4rqqrFfpe
%ea0xe9Lq-Jc9vqaqpepm0xbbG8FasPYRqj0-yi0dXdbba9pGe9
%xq-JbbG8A8frFve9Fve9Ff0dmeaabaqaciGacaGaaeqabaWaae
%aaeaaakeaacaWG4bGaey4kaSIaamyEaaaa!3934!
```

For more details on this format, and on the representation of MTEF as plain text, see the [Translator Output](https://docs.wiris.com/mathtype/en/), and [Representing MTEF as Text](https://docs.wiris.com/mathtype/en/) sections below.

## GIF image files

MTEF data is embedded into a GIF file as an Application Extension Record, which consists of a 14-byte header (Application Extension Descriptor), followed by the MTEF data. The header contains:

```
Byte Introducer = 0x21;
Byte ExtensionLabel = 0xFF;
Byte BlockSize = 0x0B;
Byte ApplicationId[8] = "MathType";
Byte AuthenticationCode[3] = "001";
```

The data follows this header and is written as a series of blocks each containing 255 bytes or less. Each block starts with a single byte count followed by the data. The end is marked as a block with length 0.

The header is unique enough that the easiest way to extract the data might be to scan the file for the 14-byte header, then expect the MTEF data blocks to follow. Properly decoding the GIF records isn't that hard either, but obviously requires you read the GIF specification.

Starting with MathType 5 for Windows, the baseline offset for the equation is also stored in the GIF file using a similar Application Extension Record as described above, but with an AuthenticationCode of "002". For details, see [Extracting Baseline Info from a GIF File](https://docs.wiris.com/mathtype/en/mathtype-office-tools/mathtype-7-for-windows-and-mac/mathtype-sdk/baselinegif).

## Clipboard and drag-and-drop

MathType registers a clipboard format with the name "MathType EF", and uses this type for MTEF data transferred via the Windows clipboard or drag-and-drop mechanisms. This type is also used in [OLE equation objects](https://docs.wiris.com/mathtype/en/).

## OLE equation objects

MTEF data is saved as the native data format of the object. Whenever an equation object is to be written to an OLE "stream", a 28-byte header is written, followed by the MTEF data. The C struct for this header is as follows:

```
struct EQNOLEFILEHDR {
 WORD cbHdr; // length of header, sizeof(EQNOLEFILEHDR) = 28 bytes
 DWORD version; // hiword = 2, loword = 0
 WORD cf; // clipboard format ("MathType EF")
 DWORD cbObject; // length of MTEF data following this header in bytes
 DWORD reserved1; // not used
 DWORD reserved2; // not used
 DWORD reserved3; // not used
 DWORD reserved4; // not used
};
```

The cf member is the return value of a call to the Windows API function RegisterClipboardFormat("MathType EF"). This type is also used in [Clipboard and Drag-and-Drop](https://docs.wiris.com/en_US/mathtype-sdk/how-mathml-is-stored-in-files-and-the-clipboard#clipboard-and-drag-and-drop-6) data transfers.

## Translator output

MathType 6.0's translators generate text in various languages, including TeX/LaTeX and MathML. MTEF is embedded in translator output much as with EPS. However, to deal with the various possible commenting conventions in languages for which translators might be written, some extra information is given in a header line. Here is an example from MathType 7 for the expression *x* + *y*, as produced by the LaTeX translator:

```
% MathType!MTEF!2!1!+-
% feaahaart1ev3aqatCvAUfeBSjuyZL2yd9gzLbvyNv2CaerbuLwBLn
% hiov2DGi1BTfMBaeXatLxBI9gBaerbd9wDYLwzYbItLDharqqtubsr
% 4rNCHbGeaGqiVu0Je9sqqrpepC0xbbL8F4rqqrFfpeea0xe9Lq-Jc9
% vqaqpepm0xbbG8FasPYRqj0-yi0dXdbba9pGe9xq-JbbG8A8frFve9
% Fve9Ff0dmeaabaqaciGacaGaaeqabaWaaeaaeaaakeaacaWG4bGaey
% 4kaSIaamyEaaaa!3935!
```

To see how this format works, let's follow through the steps used by MathType to extract MTEF from translator output when, for example, it is pasted in via the clipboard:

1. Search for a line containing "MathTypeXMTEFX" where X is any character. Both X's must be the same. X is called the "delimiter".
2. The two numbers after "MTEF" in the header line are the number of characters at the beginning and end of each comment line, surrounding a chunk of MTEF. The end-of-line is counted in the end value.
3. The final two characters on the header line are to be used in addition to the upper and lower-case alphabets and the digits to give 64 characters in which to encode the MTEF data. See [Representing MTEF as Text](https://docs.wiris.com/mathtype/en/) for details on the encoding.
4. Lines are read until a line containing the delimiter character is read. An internal limit is placed on this process so that it doesn't run forever in the case where the delimiter is missing for some reason. MTEF text should probably never exceed 100 lines.
5. The checksum follows the delimiter found in the last step which is, in turn, followed by a final delimiter. See [Representing MTEF as Text](https://docs.wiris.com/mathtype/en/) for details on the checksum.

## Representing MTEF as text

MTEF is a binary format. However, some of the file formats (e.g. EPS, translator output) in which MathType needs to save MTEF are text formats. To embed MTEF in plain text, there are two considerations that must be addressed:

- MTEF must be converted to a form that contains only legal ASCII characters;
- MTEF text must be embedded into the text in such a way as not to interfere with the meaning of the rest of text.

For example, when embedding MTEF in an EPS file, it is converted into text using an algorithm shown below and then broken into lines each prefixed by %, the EPS syntax for comments.

The binary data is converted into characters by mapping each 6 bits (taking the lowest bits in each byte first) to a single character by using the value as a 0-based index into the following string:

```
char A64[] =
 "abcdefghijklmnopqrstuvwxyz"
 "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
 "0123456789+-";
```

The value between exclamation points is a hexadecimal representation of a checksum of the MTEF data calculated by summing all the bytes into a 16-bit value. As EPS files may be edited by a human, the checksum is used to check the integrity of the MTEF data.

To convert from MTEF in its ASCII form into binary, this process is reversed. So, taking the above example as input to be converted, the first 4 characters ("feaa") convert into the first 3 bytes of MTEF as follows:

| **character** | **index** | **bits** |
| --- | --- | --- |
| 'f' | 5 | \<span>\<strong>000101\</strong>\</span> |
| 'e' | 4 | \<span>\<strong>000100\</strong>\</span> |
| 'a' | 0 | \<span>\<strong>000000\</strong>\</span> |
| 'a' | 0 | \<span>\<strong>000000\</strong>\</span> |

Putting these bits together into bytes (filling the low-order part of the byte first), we get:

\<p>\<strong>\<span>00\</span>\<span>000101\</span>\<span>0000\</span>\<span>0001\</span>\<span>000000\</span>\<span>00\</span>\</strong>\</p>

For example, the first index value (5) fills in the low 6 bits of the first byte, the second value (4) fills in the remaining 2 bits of the first byte and then the first 4 bits of the second byte, the third value (0) fills in the high 4 bits of the second byte and the 2 low bits of the third byte, and the fourth value (also 0) fills in the remaining bits of the third byte.

The binary byte stream thus reads 5, 1, 0, meaning MTEF version = 5, platform = Windows, product = MathType