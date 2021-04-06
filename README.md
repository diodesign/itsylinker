# itsylinker

Generate 64-bit RISC-V (RV64I) ELF executables from suitable ELF object files. The goal of this project is to produce a minimalist, stable linker in Rust primarily for the [Diosix](https://diosix.org) project. 

itsylinker is compatible with the `binutils ld` command-line interface. The invocation for a typical application ELF execuable would be:

```
itsylinker -o <output filename> -T <config filename> -L <search path> <list of object filenames>
```

For example to create the ELF executable `myapp` using the configuration file `link.toml`:

```
itsylinker -o myapp -T link.toml -L target/debug rt0.o main.o
```

Where `rt0.o` contains the entry point and runtime startup code, `main.o` contains the application's code, and both `.o` files are in the `target/debug` directory.

A TOML configuration file must be provided using the command line switch `-T <config filename>` to direct the linking process. Below is the `link.toml` configuration file for the example `myapp` executable:

```
/* set the entry point to the symbol _start, which should be in rt0,
   align the program to the nearest 4096 bytes when loaded into memory,
   produce a dynamically relocatable ELF executable */
[output]
entry = "_start"
alignment = 4096
dynamic_relocation = true

[section.text]
include = [ ".rt0*", ".text*", ".rodata*" ]
alignment = 8

[section.data]
include = [ ".data*" ]
alignment = 8

/* define __bss_start and __bss_end with an 8-byte alignment
   as the start and end addresses of the section.
   this can be used by the rt0 startup code to zero the bss */
[section.bss]
include = [ ".bss*" ]
alignment = 8
start_symbol = "__bss_start"
end_symbol = "__bss_end"
```

### Contact and code of conduct <a name="contact"></a>

Please [email](mailto:chrisw@diosix.org) project lead Chris Williams if you have any questions or issues to raise, wish to get involved, have source to contribute, or have found a security flaw. You can, of course, submit pull requests or raise issues via GitHub, though please consider disclosing security-related matters privately. Please also observe the Diosix project's [code of conduct](https://diosix.org/docs/conduct.html) if you wish to participate.

### Copyright and license <a name="copyright"></a>

Copyright &copy; Chris Williams, 2021. See [LICENSE](LICENSE) for distribution and use of source code and binaries.
