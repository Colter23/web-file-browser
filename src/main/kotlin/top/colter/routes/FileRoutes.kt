package top.colter.routes

import io.ktor.http.*
import io.ktor.server.application.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import top.colter.models.*
import java.io.File
import java.nio.file.Path
import kotlin.io.path.*


const val FILE_PREFIX = "%FILE%"

fun Route.fileRouting() {
    route("/file") {
        get ("/") {
            call.respond(getFolderData())
        }
        get("{path...}") {
            val pathParams = call.parameters.getAll("path") ?: return@get call.respondText("无法获取path")
            try {
                val folderData = getFolderData(pathParams)
                if (folderData.path.startsWith(FILE_PREFIX)) {
                    call.respond(Path(folderData.path.removePrefix(FILE_PREFIX)).readBytes())
                }else {
                    call.respond(folderData)
                }
            } catch (e: NoSuchFileException) {
                call.respond(HttpStatusCode.BadRequest, "查无此径: ${e.message}")
            }
        }
    }
}

fun getFolderData(paths: List<String> = listOf("")): FolderData {
    val folderList = mutableListOf<FolderInfo>()
    val fileList = mutableListOf<FileInfo>()

    // 列出真实文件夹内结构
    fun listDirs(path: Path, parentPath: String) {
        val dirs = path.listDirectoryEntries()
        dirs.forEach {
            val filePath = "${parentPath.removeSuffix("/")}/${it.name}"
            val modified = it.getLastModifiedTime().toString()
            if (it.isDirectory()) {
                folderList.add(FolderInfo(it.name, filePath, modified))
            } else {
                fileList.add(FileInfo(it.name, filePath, modified, it.fileSize(), it.extension))
            }
        }
    }

    val suchPath = "/" + paths.joinToString("/")
    if (rootFolder == null) return FolderData(suchPath, folderList, fileList)

    val pathList = mutableListOf<String>()
    pathList.addAll(paths)
    var currentFolder = rootFolder

    // 搜索可用目录
    for (path in pathList.toList()) {
        if (currentFolder is RealFolder) break
        if (currentFolder is VirtualFolder) {
            currentFolder = currentFolder.children.find { it.name == path }
            pathList.removeAt(0)
        }
    }

    if (currentFolder == null) throw NoSuchFileException(File(suchPath))

    // 获取目录信息
    if (currentFolder is RealFolder) {
        val folder = currentFolder
        val inPath = pathList.joinToString("/")
        val realPath = folder.realPath + "/" + inPath
        val path = Path(realPath)
        if (!path.isDirectory()) {
            return FolderData("$FILE_PREFIX$realPath", folderList, fileList)
        }
        listDirs(path, "${folder.path.removeSuffix("/")}/$inPath")
    } else if (currentFolder is VirtualFolder) {
        currentFolder.children.forEach {
            val modified = if (it is RealFolder) Path(it.realPath).getLastModifiedTime().toString() else ""
            folderList.add(FolderInfo(it.name, it.path, modified))
        }
    }
    return FolderData(suchPath, folderList, fileList)

}