# Process Key Sender 🦀

![Version](https://img.shields.io/badge/version-1.0.0-brightgreen) ![License](https://img.shields.io/badge/license-MIT-blue) ![Platform](https://img.shields.io/badge/platform-cross--platform-lightgrey)

Welcome to the **Process Key Sender** repository! This project offers a powerful cross-platform keystroke automation tool, built in Rust. You can send keystrokes to specific processes with customizable intervals, making it an excellent choice for accessibility, productivity, and offline gaming automation.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Configuration](#configuration)
- [Contributing](#contributing)
- [License](#license)
- [Contact](#contact)
- [Releases](#releases)

## Features

- **Cross-Platform**: Works seamlessly on Windows, macOS, and Linux.
- **Keystroke Automation**: Send keystrokes to any active process.
- **Configurable Intervals**: Set custom intervals for sending keys.
- **Command-Line Interface**: Simple and effective CLI for easy use.
- **JSON Configuration Support**: Manage settings through a straightforward JSON file.

## Installation

To get started with Process Key Sender, you need to download the latest release. Visit the [Releases section](https://github.com/raheemawan803/process-key-sender/releases) to find the appropriate file for your platform. Download and execute the file to install the tool.

## Usage

After installation, you can start using Process Key Sender from your command line. Here’s a simple example of how to use it:

```bash
process-key-sender --process "YourProcessName" --keys "a,b,c" --interval 1000
```

In this example, the tool sends the keys `a`, `b`, and `c` to the specified process every second (1000 milliseconds).

## Configuration

Process Key Sender supports configuration through a JSON file. Here’s an example of a configuration file:

```json
{
  "process": "YourProcessName",
  "keys": ["a", "b", "c"],
  "interval": 1000
}
```

Save this file as `config.json`, and you can run the tool with the following command:

```bash
process-key-sender --config config.json
```

## Contributing

We welcome contributions to improve Process Key Sender. If you would like to contribute, please follow these steps:

1. Fork the repository.
2. Create a new branch for your feature or bug fix.
3. Make your changes and commit them.
4. Push your branch to your forked repository.
5. Create a pull request to the main repository.

Please ensure your code follows the project's coding standards and includes tests where applicable.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contact

For questions or suggestions, feel free to open an issue in this repository or contact the maintainer directly.

## Releases

To stay updated with the latest features and improvements, check the [Releases section](https://github.com/raheemawan803/process-key-sender/releases) regularly. Download the latest version and start automating your keystrokes today!

---

Thank you for checking out Process Key Sender! We hope this tool enhances your productivity and makes your tasks easier. If you have any feedback or need assistance, don't hesitate to reach out. Happy automating!