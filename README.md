# Tito, el robotito (WIP)

Tito is a rust-coded robot meant to help professors that teach topics that involve programming with an automated form of grading. It is relatively simple to use, and it only requires a folder with source files to be evaluated, and a `settings.json` file to  work with.

The workflow is the following

1. Build arena file with `ru-botito build --settings ./settings.json`. If you don't know how to build the settings file, execute `ru-botito build --example-config`, which will create a `settings.json` file with a dummy problem `problem-a.sh` at the directory of execution.
2. Execute tito with arena file like `ru-botito run --arena ./arena.json --competitor <competitor>`, where competitor is composed of three strings with no spaces separated by two dots, the first field being the id, the second being the path to the folder with the files, and the third being the path to the folder to hand in results to that user. For example, `carlos:/path/to/files:/path/to/result`

Done! You will find the total grades in the execution folder as `results.json`, a json file with certain detail of execution.

## Example settings

The following is an example configuration file that creates the arena file for easier evaluation

```json
{
    "proposals": {
        "problem-a": {
        "scenarios": [
            {
            "arguments": [
                "Tito"
            ],
            "input": null,
            "max_time": 1.0,
            "max_ram": null,
            "points": 10
            }
        ],
        "solution": "./problem-a.sh",
        "language": "Shell",
        "points": 10
        }
    },
    "language_settings": {
        "Shell": {
        "pre_tools": null,
        "tool": {
            "utility": "bash",
            "temporal": false,
            "arguments": [
            "{filename}"
            ]
        },
        "extension": "sh"
        }
    }
}
```

## Supported languages

For the moment, `Rust`, `C`, `C++`, `Python2`, `Python3` and `bash` are supported, but in theory it is trivial to add a new language. Documentation in this regard will be available soon.