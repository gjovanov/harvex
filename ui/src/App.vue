<template>
  <v-app>
    <v-app-bar density="compact" color="primary">
      <v-app-bar-title>
        <router-link to="/" class="text-white text-decoration-none">
          Harvex
        </router-link>
      </v-app-bar-title>
      <template #append>
        <v-btn icon to="/upload">
          <v-icon>mdi-upload</v-icon>
        </v-btn>
      </template>
    </v-app-bar>

    <v-main>
      <v-container fluid>
        <router-view />
      </v-container>
    </v-main>
  </v-app>
</template>

<script setup lang="ts">
</script>
