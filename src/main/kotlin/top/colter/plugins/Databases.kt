package top.colter.plugins

import io.ktor.server.application.*
import org.jetbrains.exposed.sql.*


var database: Database? = null

fun Application.configureDatabases() {
    val url = environment.config.property("database.url").getString()
    val driver = environment.config.property("database.driver").getString()
    val user = environment.config.property("database.user").getString()
    val password = environment.config.property("database.password").getString()
    database = Database.connect(
        url = url,
        driver = driver,
        user = user,
        password = password
    )
}