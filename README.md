# SiftFS

A simple FUSE that renders a view of an existing directory which organizes the files in the directory into an appropriate directory structure, with custom filenames, based on a regex and format string.


## What It Looks Like

Say you have a directory with course material from a bunch of subjects (in our case, the courses are Physics 101, Mathematics 101 and Algoritms 201). Each file is a PDF file formatted as `<course name>-<lecture name>-<material number>.<filetype>`.

```bash
❯ ls ~/course-materials

algorithms-a-1.pdf  algorithms-b-3.pdf   mathematics-a-2.pdf  mathematics-d-1.pdf  physics-a-4.pdf  physics-c-1.pdf
algorithms-a-2.pdf  algorithms-c-1.pdf   mathematics-b-1.pdf  mathematics-d-2.pdf  physics-b-1.pdf  physics-c-2.pdf
algorithms-a-3.pdf  algorithms-c-2.pdf   mathematics-b-2.pdf  physics-a-1.pdf      physics-b-2.pdf  physics-c-3.pdf
algorithms-b-1.pdf  algorithms-c-3.pdf   mathematics-c-1.pdf  physics-a-2.pdf      physics-b-3.pdf  physics-c-4.pdf
algorithms-b-2.pdf  mathematics-a-1.pdf  mathematics-c-2.pdf  physics-a-3.pdf      physics-b-4.pdf
```

This is messy, and you'd like a directory structure where each course has a separate directory. This is where SiftFS comes to the rescue!

### Setting Things Up

First you need to modify 3 constants in `src/main.rs`:
1. Set `FILE_REGEX` to a regex that can identify the components of each file. Make sure you use regex groups for the pieces you need to construct the filename. Here's the one for this example: `^([A-Za-z]+)-([A-Za-z]+)-([0-9]+)\.([a-z]+)$`.
2. Set `FILE_FORMAT_STRING` to a format string where `{}` is replaced by a component of the file (this will be the match from one of the regex groups above). For our example, we'll use `{} - {} ({}).{}` where the `{}` will be replaced by the course name, lecture name, lecture number and filetype in that order.
3. Set `FILE_FORMAT_STRING_ARGS` to a comma delimited string of integers representing the group numbers of the regex groups from above that should be filled into the format string. Note that the first of these should ALWAYS be `1`. Here we retain the order of the pieces and use `1,2,3,4`.

Now we create a directory that will serve as a mount point for the new FS.

```bash
$ mkdir /tmp/course-materials-pretty
```

### Running

And finally, we run our FUSE as follows:

```bash
# You can also use the compiled binary, but I'll be using cargo run here
$ cargo run <path to original directory> <path to mount point>

# In our example this would be
$ cargo run ~/course-materials /tmp/course-materials-pretty
```

### The End Result

The FUSE makes directories using the first regex group match as the directory name and prettifies the filenames according to the `FILE_FORMAT_STRING` that we supplied. Here's the result of the `tree` command in `/tmp/course-materials-pretty`:

```bash
❯ tree
.
├── algorithms
│   ├── algorithms - a (1).pdf
│   ├── algorithms - a (2).pdf
│   ├── algorithms - a (3).pdf
│   ├── algorithms - b (1).pdf
│   ├── algorithms - b (2).pdf
│   ├── algorithms - b (3).pdf
│   ├── algorithms - c (1).pdf
│   ├── algorithms - c (2).pdf
│   └── algorithms - c (3).pdf
├── mathematics
│   ├── mathematics - a (1).pdf
│   ├── mathematics - a (2).pdf
│   ├── mathematics - b (1).pdf
│   ├── mathematics - b (2).pdf
│   ├── mathematics - c (1).pdf
│   ├── mathematics - c (2).pdf
│   ├── mathematics - d (1).pdf
│   └── mathematics - d (2).pdf
└── physics
    ├── physics - a (1).pdf
    ├── physics - a (2).pdf
    ├── physics - a (3).pdf
    ├── physics - a (4).pdf
    ├── physics - b (1).pdf
    ├── physics - b (2).pdf
    ├── physics - b (3).pdf
    ├── physics - b (4).pdf
    ├── physics - c (1).pdf
    ├── physics - c (2).pdf
    ├── physics - c (3).pdf
    └── physics - c (4).pdf

3 directories, 29 files
```

## Build

This is a simple cargo project!

```bash
$ cargo build
```
