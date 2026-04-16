# 杀戮尖塔2 存档编辑器
这是一个简单的、使用rust的、针对游戏 `Slay The Spire II` 制作的存档编辑器

原理是解析 `current_run.save` ，且能通过简单且直观的方式编辑它。

## 工作原理
它能探测多个路径，包括但不限于如下：

`~/.local/share/Steam/steamapps/compatdata/[Random ID]/pfx/drive_c/users/steamuser/AppData/Roaming/GSE Saves/2868840/remote/profile1/saves/current_run.save`

`~/.wine/drive_c/users/[User Name]/AppData/Roaming/GSE Saves/2868840/remote/profile1/saves/current_run.save`

随后软件将解析文件并暴露直观的编辑方式到GUI

## 计划功能
- **内存注入游戏** 实现游戏内编辑和即时修改
- **遗物和卡牌修改** 来进行原子级修改
- **Mods** 这个嘛，有点难哇。。。