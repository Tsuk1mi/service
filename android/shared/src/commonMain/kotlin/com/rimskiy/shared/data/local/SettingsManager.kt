package com.rimskiy.shared.data.local

import com.russhwolf.settings.Settings
import com.russhwolf.settings.get
import com.russhwolf.settings.set

class SettingsManager(private val settings: Settings) {
    companion object {
        private const val KEY_AUTH_TOKEN = "auth_token"
        private const val KEY_USER_ID = "user_id"
        private const val KEY_BASE_URL = "base_url"
    }

    var authToken: String?
        get() = try {
            val value = settings.getString(KEY_AUTH_TOKEN, "")
            if (value.isEmpty()) null else value
        } catch (e: Exception) {
            null
        }
        set(value) {
            if (value != null) {
                settings.putString(KEY_AUTH_TOKEN, value)
            } else {
                settings.remove(KEY_AUTH_TOKEN)
            }
        }

    var userId: String?
        get() = try {
            val value = settings.getString(KEY_USER_ID, "")
            if (value.isEmpty()) null else value
        } catch (e: Exception) {
            null
        }
        set(value) {
            if (value != null) {
                settings.putString(KEY_USER_ID, value)
            } else {
                settings.remove(KEY_USER_ID)
            }
        }

    var baseUrl: String?
        get() = try {
            val value = settings.getString(KEY_BASE_URL, "")
            if (value.isEmpty()) null else value
        } catch (_: Exception) {
            null
        }
        set(value) {
            if (value != null && value.isNotBlank()) {
                settings.putString(KEY_BASE_URL, value)
            } else {
                settings.remove(KEY_BASE_URL)
            }
        }

    fun clearAuth() {
        settings.remove(KEY_AUTH_TOKEN)
        settings.remove(KEY_USER_ID)
    }

    val isAuthenticated: Boolean
        get() = authToken != null
}

