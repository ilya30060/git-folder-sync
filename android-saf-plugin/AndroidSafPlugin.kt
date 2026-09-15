package com.gitfoldersync.saf

import android.app.Activity
import android.content.Intent
import android.net.Uri
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.util.concurrent.Executors

@InvokeArg
class TreeArgs {
    lateinit var uri: String
    lateinit var destination: String
}

@TauriPlugin
class AndroidSafPlugin(private val activity: Activity) : Plugin(activity) {
    private val executor = Executors.newSingleThreadExecutor()
    private var pendingPick: Invoke? = null
    private val pickRequest = 7401

    @Command
    fun pickDirectory(invoke: Invoke) {
        pendingPick = invoke
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT_TREE).apply {
            addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            addFlags(Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
            addFlags(Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION)
            addFlags(Intent.FLAG_GRANT_PREFIX_URI_PERMISSION)
        }
        activity.startActivityForResult(intent, pickRequest)
    }

    @Command
    fun importTree(invoke: Invoke) {
        val args = invoke.parseArgs(TreeArgs::class.java)
        executor.execute {
            try {
                val count = TreeSync(activity).importTree(Uri.parse(args.uri), args.destination)
                invoke.resolve(JSObject().apply { put("files", count) })
            } catch (t: Throwable) {
                invoke.reject(t.message ?: "SAF import failed")
            }
        }
    }

    @Command
    fun exportTree(invoke: Invoke) {
        val args = invoke.parseArgs(TreeArgs::class.java)
        executor.execute {
            try {
                val count = TreeSync(activity).exportTree(Uri.parse(args.uri), args.destination)
                invoke.resolve(JSObject().apply { put("files", count) })
            } catch (t: Throwable) {
                invoke.reject(t.message ?: "SAF export failed")
            }
        }
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        if (requestCode != pickRequest) return
        val call = pendingPick ?: return
        pendingPick = null
        if (resultCode != Activity.RESULT_OK || data?.data == null) {
            call.reject("cancelled")
            return
        }
        val uri = data.data!!
        val flags = data.flags and
            (Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
        try {
            activity.contentResolver.takePersistableUriPermission(uri, flags)
        } catch (_: SecurityException) {
            call.reject("Android не дал сохранить разрешение на выбранную папку")
            return
        }
        call.resolve(JSObject().apply { put("uri", uri.toString()) })
    }
}
