# The Unreal Engine 3 archive format

**NOTE:** Most of this research is based on Gildor's [UModel](https://github.com/gildor2/UEViewer/).

UE3 .u, .upk, and .umap files all share the same file format, which Stitchkit collectively refers
to as the *FArchive format.* This document outlines the structure of the format as seen in
A Hat in Time.

## Overall structure

An FArchive is made up of multiple sections:

- **Summary**
- **Name table**
- **Export table**
- **Import table**

**The summary is always located at the beginning of the archive**. The positions of other sections
can be found by looking at what's stored in the summary.

## Fundamental types

### Primitives

The unsigned integers *u8*, *u16*, *u32*, *u64*, and signed integers *i8*, *i16*, *i32*, *i64*
are available.

Additionally the notation *T*\[n] is used to signify an array of n *T*s stored back to back.

The type *magic* is used for arrays of magic bytes. Magic byte sequences must match exactly in the
file.

### *array*\<T>

Serialized Unreal `TArray` with elements of type `T`. Unlike *T*\[n], the size is flexible.

| Name | Type | Description |
| --- | :-: | --- |
| num | *u32* | Number of elements. |
| data | *T*\[num] | The elements themselves.

### *string*

Serialized Unreal `FString`.

| Name | Type | Description |
| --- | :-: | --- |
| chars | *array*\<*u8*> | The string character array. Usually terminated by a NUL byte. |

### *name*

Serialized `FName`.

| Name | Type | Description |
| --- | :-: | --- |
| index | *u32* | Index into the archive's name table. |
| serial_number | *u32* | "Serial number" used for disambiguating duplicate names. Usually represented with a suffix like `_123` in the editor. |

### *guid*

Serialized `FGuid`.

| Name | Type |
| --- | :-: |
| a | *u32* |
| b | *u32* |
| c | *u32* |
| d | *u32* |

## *Summary*

Contains overall information about the structure of the archive.

| Name | Type | Description |
| --- | :-: | --- |
| magic | *magic* | The magic bytes `c1 83 2a 9e`. |
| file_version | *u16* | Overall FArchive format version; for Hat it's 893. |
| licensee_version | *u16* | Similar to file_version but incremented by UE3 licensees. For Hat it's 5. |
| headers_size | *u32* | My best guess is that this is the collective size of the header sections, but I'm not sure. |
| package_group | *string* | - |
| package_flags | *u32* | - |
| name_count | *u32* | The number of *name*s stored in the file's name table. |
| name_offset | *u32* | The position of the name table in the file. |
| export_count | *u32* | The number of *ObjectExport*s stored in the file's export table. |
| export_offset | *u32* | The position of the export table in the file. |
| import_count | *u32* | The number of *ObjectImport*s stored in the file's import table. |
| import_offset | *u32* | The position of the import table in the file. |
| depends_offset | *u32* | - |
| - | *u32* | Unknown; mirrors headers_size. |
| - | *u32* | Unknown; always zero. |
| - | *u32* | Unknown; always zero. |
| - | *u32* | Unknown; always zero. |
| guid | *guid* | The package's globally unique identifier (GUID.) |
| generations | *array*<*GenerationInfo*> | - |
| engine_version | *u32* | - |
| cooker_version | *u32* | - |
| compression_kind | *u32* | 0 if the package is not compressed, 2 if the package is LZO-compressed. |
| compressed_chunks | *array*<*CompressedChunkPointer*> | List of pointers to compressed chunks in the package. |

### *GenerationInfo*

The purpose of this data is unknown to me.

| Name | Type |
| --- | :-: |
| export_count | *u32* |
| name_count | *u32* |
| net_object_count | *u32* |

## Compression

FArchives may be compressed. A Hat in Time archives only ever seem to use LZO compression.
The places where data is compressed are indicated by the compressed_chunks field of *Summary*.
These compressed chunk pointers point to *CompressedChunk*s, which are made up of a
*CompressedChunkHeader* and a number of *CompressedChunkBlock*s, which is dependent on the value of
uncompressed_data in a *CompressedChunkPointer* and the *CompressedChunkHeader*'s block_size field.

### *CompressedChunkPointer*

| Name | Type | Description |
| --- | :-: | --- |
| uncompressed_offset | *u32* | Where the compressed data should land after decompression. |
| uncompressed_size | *u32* | How much data there should be after decompression. |
| compressed_offset | *u32* | Where the *CompressedChunk* is in the file. |
| compressed_size | *u32* | Total size of the *CompressedChunk*. |

### *CompressedChunk*

| Name | Type | Description |
| --- | :-: | --- |
| header | *CompressedChunkHeader* | - |
| blocks | *CompressedChunkBlock*\[n] | n = (header.sum.uncompressed_size + header.block_size - 1) / header.block_size |

### *CompressedChunkHeader*

| Name | Type | Description |
| --- | :-: | --- |
| magic | *magic* | Magic bytes `c1 83 2a 9e`. |
| block_size | *u32* | The size of each *CompressedChunkBlock*. |
| sum | *CompressedChunkBlock* | Sum of all *CompressedChunkBlock*'s sizes. |

### *CompressedChunkBlock*

| Name | Type |
| --- | :-: |
| compressed_size | *u32* |
| uncompressed_size | *u32* |

## *NameTable*

| Name | Type |
| --- | :-: |
| names | *array*<*NameTableEntry*> |

### *NameTableEntry*

| Name | Type | Description |
| --- | :-: | --- |
| name | *string* | The actual name string. |
| flags | *u64* | Those seem to be object flags, but I don't know what they mean at all. The value is always 0x0007001000000000. |

## *ExportTable*

| Name | Type |
| --- | :-: |
| exports | *array*<*ObjectExport*> |

### *ObjectExport*

| Name | Type | Description |
| --- | :-: | --- |
| class_index | *PackageObjectIndex* | This object's class. |
| outer_index | *PackageObjectIndex* | This object's outer object. Not sure what it means when this is negative. |
| package_index | *PackageObjectIndex* | Unknown.
| object_name | *name* | The name of this object.
| archetype | *u32* | - |
| flags | *u64* | - |
| serial_size | *u32* | The size of this object's serialized data. |
| serial_offset | *u32* | The position of the object's serialized data in the package file. |
| export_flags | *u32* | - |
| net_object_count | *u32* | - |
| guid | *guid* | - |
| - | *u32* | - |

### *PackageObjectIndex*

The index of an object within this package. When it's is positive, it refers to one of the package's
exports. When it's negative, it refers to one of the package's imports. When it's zero it refers
to `Class`.

| Name | Type |
| --- | :-: |
| index | *u32* |

## *ImportTable*

| Name | Type |
| --- | :-: |
| imports | *array*<*ObjectImport*> |

### *ObjectImport*

| Name | Type | Description |
| --- | :-: | --- |
| package | *name* | The package of the imported object. |
| class_name | *name* | The name of the object's class. |
| package_index | *PackageObjectIndex* | My best guess is that this specifies which *PackageObjectIndex* the imported object should occupy within this archive. |
| object_name | *name* | The name of the imported object. |
