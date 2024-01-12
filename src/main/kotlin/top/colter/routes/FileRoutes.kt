package top.colter.routes

import io.ktor.server.application.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import top.colter.models.*
import kotlin.io.path.*


const val FILE_PREFIX = "%FILE%"

fun Route.fileRouting() {
    route("/file") {
        get ("/") {
            call.respond(getFolderData(rootFolder))
        }
        get("{path...}") {
            val pathParams = call.parameters.getAll("path") ?: return@get call.respondText("无法获取path")
            val folderData = getFolderData(rootFolder, pathParams)
            if (folderData.path.startsWith(FILE_PREFIX)) {
                call.respond(Path(folderData.path.removePrefix(FILE_PREFIX)).readBytes())
            }else {
                call.respond(folderData)
            }
        }
    }
}

fun getFolderData(rootFolder: VirtualFolder, paths: List<String>? = null): FolderData {
    val folderList = mutableListOf<FolderInfo>()
    val fileList = mutableListOf<FileInfo>()

    var currentFolder = rootFolder
    paths?.forEachIndexed { index, p ->
        val folder = currentFolder.children.find { it.name == p }
        if (folder is RealFolder) {
            val inPath = if (paths.size == 1) "" else "/" + paths.drop(index + 1).joinToString("/")
            val realPath = folder.realPath + inPath
            val path = Path(realPath)
            if (!path.isDirectory()) {
                return FolderData("$FILE_PREFIX$realPath", folderList, fileList)
            }
            val dirs = path.listDirectoryEntries()
            dirs.forEach {
                val filePath = "${folder.path}$inPath/${it.name}"
                if (it.isDirectory()) {
                    folderList.add(FolderInfo(it.name, filePath, it.getLastModifiedTime().toString()))
                }else {
                    fileList.add(FileInfo(it.name, filePath, it.getLastModifiedTime().toString(), it.fileSize(), it.extension))
                }
            }
            return FolderData("/" + path.joinToString("/"), folderList, fileList)
        }else if (folder is VirtualFolder) {
            currentFolder = folder
        }
    }
    currentFolder.children.forEach {
        val modified = if (it is RealFolder) Path(it.realPath).getLastModifiedTime().toString() else ""
        folderList.add(FolderInfo(it.name, it.path, modified))
    }

    return FolderData("/" + paths?.joinToString("/"), folderList, fileList)
}
