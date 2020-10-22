# Tito, el robotito (WIP)

Tito is a rust-coded robot meant to help professors that teach topics that involve programming with an automated form of grading. It is relatively simple to use, and it only requires a folder with source files to be evaluated, and a `settings.json` file to  work with.

The workflow is the following

1. Build arena file with `ru-botito build --settings ./settings.json`
2. Execute tito with arena file like `ru-botito run --arena ./arena.json --competitor <competitor>`, where competitor is composed of three strings with no spaces separated by two dots, the first field being the id, the second being the path to the folder with the files, and the third being the path to the folder to hand in results to that user. For example, `carlos:/path/to/files:/path/to/result`

Done! You will find the total grades in the execution folder as `results.json`, a json file with certain detail of execution.

## Supported languages

For the moment, `Rust`, `C`, `C++`, `Python2`, `Python3` and `bash` are supported, but in theory it is trivial to add a new language. Documentation in this regard will be available soon.