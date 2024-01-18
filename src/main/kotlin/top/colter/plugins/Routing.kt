package top.colter.plugins

import io.ktor.server.application.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import top.colter.routes.fileRouting
import top.colter.routes.mappingRouting


fun Application.configureRouting() {
    routing {
        route("/api") {
            get("/") {
                call.respondText("Web File Browser Server")
            }
            fileRouting()
            mappingRouting()
        }
    }
}
