<h2 align="center">
  <a href="https://runebook.co" target="blank_">
    <img src="https://runebook.co/images/wand_horizontal.png" width="75%" align="center" />
    <br/>
    <br/>
  </a>
</h2>

## 🔮 Official Runebook CLI

### ⚡️ Getting Started

Requires Homebrew.

```bash
brew tap runebookco/wand
brew install wand
```

### 🧑‍💻 Login

```bash
wand login
```

Logs into Runebook and stores your credentials locally.

### 🛟 Agent Install

```bash
wand install
```

Installs the Runebook Kubernetes Agent. This requires `kubectl` and that it has
access to the cluster you want to install into.

It will first show you a diff of the resource it will create and asks you to
confirm before you continue.
