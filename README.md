# Slay The Spire II Profile Info Editor
This is a simple rust app for editing profile of  `Slay The Spire II`

It deconstructs profile files(`current_run.save`) and able to edit it with simple and intuitive ways.

## How does it work
It detects multiple paths, including:
`~/.local/share/Steam/steamapps/compatdata/[Random ID]/pfx/drive_c/users/steamuser/AppData/Roaming/GSE Saves/2868840/remote/profile1/saves/current_run.save`

`~/.wine/drive_c/users/[User Name]/AppData/Roaming/GSE Saves/2868840/remote/profile1/saves/current_run.save`

and so on.

Then it will deconstructs these JSON files and expose intuitive buttons for easily controlling.

## Future feats
- **Memory Injection** for in-game and immediate cracking.
- **Relics and Cards Modification** for atom degree editing.
- **Mods** , Maybe... It is not that easy