# Rust OneNote® File Parser

A parser for Microsoft OneNote® files implemented in Rust.

## Status

Work in progress. Right now it can parse most of OneNote file contents but only
if the files are in the FSSHTTP packaging format [\[MS-ONESTORE\] 2.8]. OneNote files
as created and stored by the OneNote 2016 desktop application are not yet
supported.

## Goals

- Read OneNote files available through both the OneNote 2016 application as
  well as through OneDrive download
- Convert OneNote notebooks and sections into HTML (see the [one2html] project)

## Non-Goals

- The ability to write OneNote files

## Architecture

The code organization and architecture follows the OneNote file format which is
build from several layers of encodings:

- `fsshttpb/`: This implements the FSSHTTP binary packaging format as specified
  in [\[MS-FSSHTTPB\]: Binary Requests for File Synchronization via SOAP Protocol].
  This is the lowest level of the file format and specifies how objects and their
  relationships are encoded (and decoded) from a binary stream (in our case a file).
- `onestore/`: This implements the OneStore format as specified in
  [\[MS-ONESTORE\]: OneNote Revision Store File Format] which describes how a
  OneNote revision store file (also called OneStore) containing all OneNote objects
  is stored in a FSSHTTP binary packaging file. This also includes the file header
  ([\[MS-ONESTORE\] 2.8]) and then how the OneNote revision store is built from the
  FSSHTTP objects and revisions ([\[MS-ONESTORE\] 2.7]).
- `one/`: This implements the OneNote file format as specified in [\[MS-ONE\]:
  OneNote File Format]. This specifies how objects in a OneNote file are parsed
  from a OneStore revision file.
- `onenote/`: This finally implements an API that provides access to the data
  stored in a OneNote file. It parses the FSSHTTPB data, the revision store
  data and then constructs the objects contained by the OneNote file. This includes
  resolving all references, e.g. looking up page's paragraphs.

## Related Resources

- [\[MS-ONESTORE\]: OneNote Revision Store File Format]
- [\[MS-ONE\]: OneNote File Format]
- [\[MS-FSSHTTPB\]: Binary Requests for File Synchronization via SOAP Protocol]
- [LibMsON]: A work in progess OneNote® revision store file parser in C++
- [FSSHTTP - parser tools for protocol FSSHTTP/B/D]: A FSSHTTPB data parser

## Disclaimer

This project is neither related to nor endorsed by Microsoft in any way. The
author does not have any affiliation with Microsoft.

[\[MS-ONESTORE\] 2.7]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/189f186c-84ea-4892-afca-633c22bf9389
[\[MS-ONESTORE\] 2.8]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/c65f7aa8-4f0e-45dc-aabd-96db97cedbd4
[\[MS-ONESTORE\]: OneNote Revision Store File Format]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-onestore/ae670cd2-4b38-4b24-82d1-87cfb2cc3725
[\[MS-ONE\]: OneNote File Format]: https://docs.microsoft.com/en-us/openspecs/office_file_formats/ms-one/73d22548-a613-4350-8c23-07d15576be50
[\[MS-FSSHTTPB\]: Binary Requests for File Synchronization via SOAP Protocol]: https://docs.microsoft.com/en-us/openspecs/sharepoint_protocols/ms-fsshttpb/f59fc37d-2232-4b14-baac-25f98e9e7b5a
[LibMsON]: https://github.com/blu-base/libmson/
[FSSHTTP - parser tools for protocol FSSHTTP/B/D]: https://github.com/marx-yu/FSSHTTP
[one2html]: https://github.com/msiemens/one2html