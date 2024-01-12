package top.colter

import io.ktor.server.application.*
import top.colter.plugins.*

fun main(args: Array<String>) {
    io.ktor.server.netty.EngineMain.main(args)
}

fun Application.module() {
    configureDatabases()
    configureSerialization()
    configureRouting()
    configureCORS()
}
