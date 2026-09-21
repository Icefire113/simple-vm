souce text -> tokenizing

from the tokens we should extract the locations (colon + identifier pairs) require that all have a `:main` block which will be
placed at start of the program.

```sasm
:main
blah blah
jmp loc_1

:loc_2
exit

:loc_1
jmp loc_2
```

Then we have to turn the tokens into a valid set of instructions, problems:

- how do we know where we are jumping?
  - Some way to turn the `jmp loc` into a `jmp <addr>`, requires that we know the location of `loc`
  - Turn tokens into instructions and keep metadata about locations aswell?
