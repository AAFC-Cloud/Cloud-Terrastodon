# v0.4.0

- Fix PIM role activation happening twice when two role assignments present for the same role
- Add `tf plan` for processed folder action
- Add default attribute remover during reflow when processing generated HCL

# v0.3.0

- Fix policy remediation not providing scope leading to 0 resources being remediated
- Add `cloud_terrastodon copy-results ./whatever` command

# v0.2.0

- Fix terminal colours in default terminal opened when double clicking the exe
    - https://stackoverflow.com/questions/78741673/colors-not-working-on-default-terminal-for-release-rust-exe/78741674#78741674
- Add app icon
- Clean up non-interactive usage scenarios (see: `cloud_terrastodon --help`)
- Linux (Ubuntu) working
- First GitHub release

# v0.1.1

- Fix "Justification:" prompt not showing when activating PIM roles