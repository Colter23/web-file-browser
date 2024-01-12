package top.colter.plugins

import io.ktor.server.application.*
import io.ktor.server.plugins.cors.routing.*


// https://ktor.io/docs/cors.html
fun Application.configureCORS() {
    install(CORS) {
        anyHost()
    }
}