# Stitchkit

The Hat in Time modder's stitching toolkit.

Stitchkit is a set of tools for building Hat in Time mods without having to use UE3's
commandlets, which take ages to do common actions such as compiling scripts or cooking packages.

Stitchkit currently provides the following tools:

- `ardump` - extract information about Unreal archives
- `objdump` - extract information about serialized UObjects

The project is currently in the research phase, where I'm researching how Unreal archives are put
together and how various fundamental reflection types are serialized.

The end goal is to have at least the following:

- UnrealScript compiler
  - The goal is to build a compiler that is meant to replace the one in vanilla Unreal,
    sporting better error messages and blazingly fast compilation.

## Crates in this repository

- `stitchkit` - CLI that ties everything together
- `stitchkit-core` - core types, binary serialization support
  - `stitchkit-core-derive` - derive macros for serialization
- Handling the FArchive binary format (.u, .upk, .umap)
  - `stitchkit-archive` - core structure of archives (sections)
  - `stitchkit-reflection-types` - reflection objects (`Class` et al.)
