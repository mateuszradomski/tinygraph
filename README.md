# Tinygraph

A simpler and more cut down version of Grafana for your self-hosting needs

# Parts

- Back: A simple application that runs in the background, periodically gathering data and saving that out to disk.
- Front: Simple webpage that gets all that data and displays it

Back:
 - Saves to disk in a custom binary format
 - Customizable e.g. delay between data gathers, item count in the array

Front:
 - Renders graphs and other gizmos into SVGs
 - Use as a simple thing as you can
  - good => Preact + htm sourced from CDN
  - better => Vanilla JS

# Output binary format [mini spec]

The binary is in stored in little endian format and is of the following structure

```
+-------+---------+-----------------+---------------+
|       |         |                 |               |
| magic | version | container count | ...containers |
|       |         |                 |               |
+-------+---------+-----------------+---------------+
```

- magic: 4 byte value equal to "TGPH"
- version: 1 byte value indicating the version of the format (most probably always 1, but good to have it)
- container count: 2 byte value equal to the number of unique containers that follow

A container is of the following structure

```
+------+--------------+---------------+-------------+
|      |              |               |             |
| name | element type | element count | ...elements |
|      |              |               |             |
+------+--------------+---------------+-------------+
```

- name: string
- element type: 1 byte indicating what type of data is stored in the elements
 - `ELEMENT_TYPE_U32` = 1
 - `ELEMENT_TYPE_FLOAT32` = 2
 - `ELEMENT_TYPE_STRING` = 3
- element count: 4 byte value equal to the number of unique elements that follow

String are encoded the following way
```
+--------+-------------+
|        |             |
| length | ASCII bytes |
|        |             |
+--------+-------------+
```

- length: variable-length integer
- ASCII bytes: length count bytes being the string (NOT null terminated)

Encoding of `length`:

```
|              Value | Bytes Used |                                  Format |
|:-------------------|:-----------|:----------------------------------------|
| >= 0 && <= 254     | 1          | uint8_t                                 |
| >= 255 && <= 65536 | 3          | 0xff followed by the number as uint16_t |
```
