# 🤝 Contributing to FM26 Mod Manager

Thanks for your interest in improving **FM26 Mod Manager!**  
This project exists to make Football Manager modding easier, cleaner, and safer — and your contributions help keep it that way.

---

## 🧭 How to Contribute

There are many ways to help:

- 🐛 **Report Bugs:** Found something broken? Open a [GitHub Issue](../../issues) with clear steps to reproduce.
- 💡 **Suggest Features:** Have an idea for a new feature or quality-of-life improvement? Submit a feature request.
- 🧰 **Improve Code:** Submit pull requests (PRs) to fix bugs, refactor code, or enhance UI.
- 📘 **Improve Docs:** Update or clarify the README, usage guide, or inline comments.

---

## 🧩 Development Setup

1. **Clone the Repository**

   ```bash
   git clone https://github.com/justinlevinedotme/FMMLoader-26.git
   cd FMMLoader-26 
   ```

2. **Create a Virtual Environment**

   ```bash
   python -m venv venv
   source venv/bin/activate  # macOS/Linux
   venv\Scripts\activate     # Windows
   ```

3. **Install Requirements**

   ```bash
   pip install -r src/requirements.txt
   ```

4. **Run the App**

   ```bash
   python fm26_mod_manager_gui.py
   ```

5. **(Optional) Build Executables**

   ```bash
   pyinstaller --noconfirm --windowed --onefile --name "FMMLoader26" src/fmmloader26.py
   ```

---

## 🧪 Pull Request Guidelines

When submitting a PR:

- Create a new branch:

  ```bash
  git checkout -b fix/detect-path-bug
  ```

- Keep commits focused and descriptive.
- Include screenshots for UI changes when possible.
- Make sure your code passes basic tests and doesn’t break existing features.

---

## ⚙️ Code Style

- Follow **PEP8** conventions.
- Use clear, descriptive variable names.
- Keep GUI labels and messages consistent with the app’s tone.
- Avoid hardcoded paths; use `os.path` or `pathlib` instead.

---

## 🧼 Testing Mods Safely

If you’re working with `.fmf` or `.zip` mods:

- Use a **test copy** of your Football Manager data directory.
- Always verify the **Apply Order** process does not overwrite core game files.
- Use the **Rollback** feature before pushing major changes.

---

## 🪪 License & Attribution

All contributions are covered under the same license as the main project:
**[CC BY-NC-SA 4.0 International](https://creativecommons.org/licenses/by-nc-sa/4.0/)**

You retain copyright to your own contributions,
but by submitting them, you agree to share under these same terms.

---

## 💬 Need Help?

If you’re unsure about something, open a **discussion** or start a thread in the project’s **Issues** tab. You can also visit the [FM Match Lab Discord](https://discord.gg/QCW7QhWdAs)
We’re friendly, and good documentation starts with good questions.

---

<p align="center">
  <em>Thank you for helping make FM26 modding more accessible for everyone.</em>
</p>
