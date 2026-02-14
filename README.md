# Simply Writer

用以解决没有一个可以操作纯文本的 Word 的问题。  
银晓觉得黑底等宽彩字的代码编辑器不是拿来写文的，又拉不下脸用 Word 写 docx。

具有两个版本：前端版本，使用 Web UI 的后端版本。

- `frontend-standalone` 为独立运行的前端版本，它只有一个 `html` 文件
- `frontend-web` & `backend` 为带后端的 **Web UI** 版本，如果需要**无感保存**就用这个，同样也仅有一个可执行文件

后端版本实现功能较为完善，**推荐使用**。

## 后端版本参数说明

```text
Usage:
  simply-writer.exe [OPTIONS] [PATH]

Arguments:
  [PATH]  要编辑的文本文件路径（如果文件不存在，程序会在保存时尝试创建）

Options:
  -p, --port <PORT>          监听端口 [默认: 3000]
  -e, --encoding <ENCODING>  文件编码 [默认: utf-8] [支持的值: utf-8, gbk]
  -h, --help                 打印帮助信息
  -V, --version              显示版本号
```
