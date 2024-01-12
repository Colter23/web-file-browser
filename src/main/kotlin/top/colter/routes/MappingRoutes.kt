package top.colter.routes

import io.ktor.http.*
import io.ktor.server.application.*
import io.ktor.server.request.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import kotlinx.coroutines.runBlocking
import top.colter.database.MappingService
import top.colter.models.PathMapping
import top.colter.models.RealFolder
import top.colter.models.VirtualFolder
import top.colter.plugins.database


// 根目录
val rootFolder = VirtualFolder(
    name = "root",
    path = ""
)

fun Route.mappingRouting() {

    val mappingService = MappingService(database!!)

    runBlocking{
        val mappings = mappingService.getAllMapping()
        mappings.forEach {
            addMountPath(rootFolder, it)
        }
    }

    route("/mapping") {
        get {
            call.respond(mappingService.getAllMapping())
        }
        get("/root") {
            call.respond(rootFolder)
        }
        post {
            val mapping = call.receive<PathMapping>()
            val allMapping = mappingService.getAllMapping()

            val path = mapping.mountPath

            val parentPath = path.dropLast(path.length - path.indexOfLast { it == '/' })
            allMapping.forEach {
                if (it.mountPath == path) return@post call.respondText("路径映射重复")
                if (it.mountPath == parentPath) return@post call.respondText("路径映射冲突")
            }
            val id = mappingService.create(mapping)
            addMountPath(rootFolder, mapping)
            call.respond(HttpStatusCode.Created, id)
        }
        delete("/{id}") {
            val id = call.parameters["id"]?.toInt() ?: throw IllegalArgumentException("Invalid ID")
            mappingService.delete(id)
            call.respond(HttpStatusCode.OK)
        }
    }
}

fun addMountPath(rootFolder: VirtualFolder, mapping: PathMapping) {
    var currentFolder = rootFolder
    val mountPathList = mapping.mountPath.removePrefix("/").split("/")
    mountPathList.dropLast(1).forEach { pathName ->
        var folder = currentFolder.children.find { it.name == pathName } as VirtualFolder?
        if (folder == null) {
            folder = VirtualFolder(pathName, "${currentFolder.path}/$pathName")
            currentFolder.children.add(folder)
        }
        currentFolder = folder
    }
    val lastPath = mountPathList.last()
    currentFolder.children.add(RealFolder(lastPath, "${currentFolder.path}/$lastPath", mapping.folderPath))
}