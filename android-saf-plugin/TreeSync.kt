package com.gitfoldersync.saf

import android.content.Context
import android.net.Uri
import androidx.documentfile.provider.DocumentFile
import java.io.File
import java.io.FileInputStream
import java.io.FileOutputStream

class TreeSync(private val context: Context) {
    private fun root(uri: Uri): DocumentFile =
        DocumentFile.fromTreeUri(context, uri) ?: error("Invalid SAF tree")

    fun importTree(uri: Uri, destination: String): Int {
        val dst = File(destination)
        dst.mkdirs()
        clearLocal(dst, keepGit = true)
        return copyFromSaf(root(uri), dst)
    }

    fun exportTree(uri: Uri, source: String): Int {
        val dst = root(uri)
        clearSaf(dst)
        return copyToSaf(File(source), dst)
    }

    private fun clearLocal(dir: File, keepGit: Boolean) {
        dir.listFiles()?.forEach { child ->
            if (keepGit && child.name == ".git") return@forEach
            child.deleteRecursively()
        }
    }

    private fun clearSaf(dir: DocumentFile) {
        dir.listFiles().forEach { it.delete() }
    }

    private fun copyFromSaf(src: DocumentFile, dst: File): Int {
        var count = 0
        src.listFiles().forEach { item ->
            val name = item.name ?: return@forEach
            val target = File(dst, name)
            if (item.isDirectory) {
                target.mkdirs()
                count += copyFromSaf(item, target)
            } else if (item.isFile) {
                context.contentResolver.openInputStream(item.uri).use { input ->
                    requireNotNull(input) { "Cannot read $name" }
                    FileOutputStream(target).use { output -> input.copyTo(output) }
                }
                count++
            }
        }
        return count
    }

    private fun copyToSaf(src: File, dst: DocumentFile): Int {
        if (!src.exists()) return 0
        var count = 0
        src.listFiles()?.forEach { file ->
            if (file.name == ".git") return@forEach
            if (file.isDirectory) {
                val child = dst.createDirectory(file.name)
                    ?: error("Cannot create directory ${file.name}")
                count += copyToSaf(file, child)
            } else if (file.isFile) {
                val child = dst.createFile("application/octet-stream", file.name)
                    ?: error("Cannot create file ${file.name}")
                context.contentResolver.openOutputStream(child.uri, "w").use { output ->
                    requireNotNull(output) { "Cannot write ${file.name}" }
                    FileInputStream(file).use { input -> input.copyTo(output) }
                }
                count++
            }
        }
        return count
    }
}
