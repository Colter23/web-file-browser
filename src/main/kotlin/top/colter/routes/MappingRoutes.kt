package top.colter.routes

import io.ktor.http.*
import io.ktor.server.application.*
import io.ktor.server.request.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import kotlinx.coroutines.runBlocking
import top.colter.database.MappingService
import top.colter.models.Folder
import top.colter.models.PathMapping
import top.colter.models.RealFolder
import top.colter.models.VirtualFolder
import top.colter.plugins.database
import kotlin.io.path.Path
import kotlin.io.path.exists


// 根目录
var rootFolder: Folder? = null

fun Route.mappingRouting() {

    val mappingService = MappingService(database!!)

    runBlocking{
        val mappings = mappingService.getAllMapping()
        mappings.forEach {
            addMountPath(it)
        }
    }

    route("/mapping") {
        get {
            call.respond(mappingService.getAllMapping())
        }
        get("/root") {
            call.respond(rootFolder ?: "null")
        }
        post {
            val mapping = call.receive<PathMapping>()
            val allMapping = mappingService.getAllMapping()

            val path = mapping.mountPath

            if (path == "/") {
                if (allMapping.isNotEmpty()) return@post call.respondText("路径映射冲突", status = HttpStatusCode.BadRequest)
            }else {
                allMapping.forEach {
                    if (it.mountPath == path) return@post call.respondText("路径映射重复", status = HttpStatusCode.BadRequest)
                    if (path.startsWith(it.mountPath)) return@post call.respondText("路径映射冲突", status = HttpStatusCode.BadRequest)
                }
            }
            if (!Path(mapping.folderPath).exists())
                return@post call.respondText("查无此径: ${mapping.folderPath}", status = HttpStatusCode.BadRequest)

            addMountPath(mapping)
            val id = mappingService.create(mapping)
            call.respond(HttpStatusCode.Created, id)
        }
        delete("/{id}") {
            val id = call.parameters["id"]?.toInt() ?: throw IllegalArgumentException("Invalid ID")
            mappingService.delete(id)

            rootFolder = null
            val mappings = mappingService.getAllMapping()
            mappings.forEach {
                addMountPath(it)
            }

            call.respond(HttpStatusCode.OK)
        }
    }
}

fun addMountPath(mapping: PathMapping) {
    if (rootFolder is RealFolder) return
    if (mapping.mountPath == "/") {
        if (rootFolder == null) rootFolder = RealFolder("root", "/", mapping.folderPath)
        return
    } else {
        if (rootFolder == null) rootFolder = VirtualFolder("root", "/")
    }

    var currentFolder = rootFolder as VirtualFolder
    // 添加虚拟文件夹
    val mountPathList = mapping.mountPath.removePrefix("/").split("/")
    mountPathList.dropLast(1).forEach { pathName ->
        var folder = currentFolder.children.find { it.name == pathName } as VirtualFolder?
        if (folder == null) {
            folder = VirtualFolder(pathName, "${currentFolder.path.removeSuffix("/")}/$pathName")
            currentFolder.children.add(folder)
        }
        currentFolder = folder
    }
    // 添加真实文件夹
    val lastPath = mountPathList.last()
    currentFolder.children.add(RealFolder(lastPath, "${currentFolder.path.removeSuffix("/")}/$lastPath", mapping.folderPath))
}