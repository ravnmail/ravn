<script lang="ts" setup>
import { ScrollArea } from '~/components/ui/scroll-area'
import AttachmentList from '~/components/Ravn/AttachmentList.vue'
import type { ConversationDetail } from '~/types/conversation'
import type { EmailAddress, Attachment } from '~/types/email'
import EmptyState from '~/components/ui/empty/EmptyState.vue'
import { Badge } from '~/components/ui/badge'

const props = defineProps<{
  conversation: ConversationDetail
}>()

// Aggregate unique members from all messages
const members = computed(() => {
  const memberMap = new Map<string, EmailAddress>()

  props.conversation.messages.forEach((message) => {
    // Add from
    if (message.from?.address) {
      memberMap.set(message.from.address.toLowerCase(), message.from)
    }

    // Add to
    message.to?.forEach((addr) => {
      if (addr.address) {
        memberMap.set(addr.address.toLowerCase(), addr)
      }
    })

    // Add cc
    message.cc?.forEach((addr) => {
      if (addr.address) {
        memberMap.set(addr.address.toLowerCase(), addr)
      }
    })

    // Add bcc
    message.bcc?.forEach((addr) => {
      if (addr.address) {
        memberMap.set(addr.address.toLowerCase(), addr)
      }
    })

    // Add reply_to
    if (message.reply_to?.address) {
      memberMap.set(message.reply_to.address.toLowerCase(), message.reply_to)
    }
  })

  return Array.from(memberMap.values()).sort((a, b) =>
    (a.name || a.address).localeCompare(b.name || b.address)
  )
})

// Convert AttachmentInfo to Attachment type and deduplicate by content hash
const allAttachments = computed<Attachment[]>(() => {
  const attachmentMap = new Map<string, Attachment>()

  // Deduplicate by hash (keep first occurrence of each unique file)
  props.conversation.attachments.forEach((att) => {
    // Use hash as deduplication key - identical files will have the same hash
    const key = att.hash || att.id // Fallback to id if hash is empty

    if (!attachmentMap.has(key)) {
      attachmentMap.set(key, {
        id: att.id,
        email_id: att.email_id,
        filename: att.filename,
        content_type: att.content_type,
        size: att.size,
        is_inline: att.is_inline,
        is_cached: att.is_cached,
        full_path: att.cache_path,
      })
    }
  })

  return Array.from(attachmentMap.values())
})

// Separate normal and inline attachments
const normalAttachments = computed(() =>
  allAttachments.value.filter(att => !att.is_inline)
)

const inlineAttachments = computed(() =>
  allAttachments.value.filter(att => att.is_inline)
)

const accountId = computed(() => props.conversation.messages[0]?.account_id)

</script>

<template>
  <div class="h-screen border-l border-border bg-surface">
    <div class="px-4 py-3 border-b border-border">
      <h2 class="text-lg font-semibold text-primary">
        Details
      </h2>
    </div>
    <ScrollArea class="flex-1">
      <div class="px-4 py-4 space-y-6">
        <div>
          <h3 class="text-sm font-semibold text-muted-foreground uppercase tracking-wide mb-3 flex items-center gap-2">
            <span>Members</span>
            <Badge size="sm">{{ members.length }}</Badge>
          </h3>
          <div class="flex flex-col gap-3">
            <div
              v-for="member in members"
              :key="member.address"
              class="flex items-center gap-2"
            >
              <RavnAvatar
                :account-id="accountId"
                :email="member.address"
                :name="member.name"
                class="shrink-0"
                size="lg"
              />
              <div class="flex-1 min-w-0">
                <div class="text-sm font-medium text-primary truncate">
                  {{ member.name || member.address }}
                </div>
                <div
                  v-if="member.name"
                  class="text-xs text-muted-foreground truncate"
                >
                  {{ member.address }}
                </div>
              </div>
            </div>
          </div>
        </div>
        <div
          v-if="normalAttachments.length > 0 || inlineAttachments.length > 0"
          class="flex flex-col gap-6"
        >
          <div v-if="normalAttachments.length > 0">
            <h3 class="text-sm font-semibold text-muted-foreground uppercase tracking-wide mb-3 flex items-center gap-2">
              <span>Attachments</span>
              <Badge size="sm">{{ normalAttachments.length }}</Badge>
            </h3>
            <AttachmentList :attachments="normalAttachments"/>
          </div>
          <div v-if="inlineAttachments.length > 0">
            <h3 class="text-sm font-semibold text-muted-foreground uppercase tracking-wide mb-3 flex items-center gap-2">
              <span>Inline Attachments</span>
              <Badge size="sm">{{ inlineAttachments.length }}</Badge>
            </h3>
            <AttachmentList :attachments="inlineAttachments"/>
          </div>
        </div>
      </div>
    </ScrollArea>
  </div>
</template>
