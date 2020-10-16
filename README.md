# Rust OneNote® File Parser

A parser for Microsoft OneNote® files implemented in Rust.

## Status

Work in progress. Right now it can parse most of OneNote file contents but only
if the files are in the FSSHTTP packaging format [[MS-ONESTORE 2.8]]. OneNote files
as created and stored by the OneNote 2016 desktop application are not yet
supported.

## Goals

- Read OneNote files available through both the OneNote 2016 application as
  well as through OneDrive download
- Convert OneNote notebooks and sections into HTML

## Non-Goals

- The ability to write OneNote files

## Related Resources

- [\[MS-ONESTORE\]: OneNote Revision Store File Format]
- [\[MS-ONE\]: OneNote File Format]
- [\[MS-FSSHTTPB\]: Binary Requests for File Synchronization via SOAP Protocol]
- [LibMsON]: A work in progess OneNote® revision store file parser in C++
- [FSSHTTP - parser tools for protocol FSSHTTP/B/D]: A FSSHTTPB data parser

## Disclaimer

This project is neither related to nor endorsed by Microsoft in any way. The
author does not have any affiliation with Microsoft.

[MS-ONESTORE 2.8]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/c65f7aa8-4f0e-45dc-aabd-96db97cedbd4
[\[MS-ONESTORE\]: OneNote Revision Store File Format]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/ae670cd2-4b38-4b24-82d1-87cfb2cc3725
[\[MS-ONE\]: OneNote File Format]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/73d22548-a613-4350-8c23-07d15576be50
[\[MS-FSSHTTPB\]: Binary Requests for File Synchronization via SOAP Protocol]: https://docs.microsoft.com/en-us/openspecs/sharepoint_protocols/ms-fsshttpb/f59fc37d-2232-4b14-baac-25f98e9e7b5a
[LibMsON]: https://github.com/blu-base/libmson/
[FSSHTTP - parser tools for protocol FSSHTTP/B/D]: https://github.com/marx-yu/FSSHTTP