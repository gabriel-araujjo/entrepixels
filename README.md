# Entrepixels

A simple steganography tool to hide and show messages from bitmap images

```
Steganography tool
Options:
  -m, --message <message>    - specifies the message to be hidden into image.
                                 Mandatory in hide command
  -o, --output <destiny>     - sets the destiny output file.
                                 Default: stdout
  -i, --input <input_file>   - sets the input image.
                                 Default: stdin
Commands:
  show                       - shows a message hidden in image
  hide                       - hide a message into an image

Usage:
  entrepixels show [-i <input>] [-o <output>]
  entrepixels hide -m <message> [-i <input>] [-o <output>]
```