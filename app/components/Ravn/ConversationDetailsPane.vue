<script lang="ts" setup>
import AttachmentList from '~/components/Ravn/AttachmentList.vue'
import { Badge } from '~/components/ui/badge'
import { Button } from '~/components/ui/button'
import { ScrollArea } from '~/components/ui/scroll-area'
import { SimpleTooltip } from '~/components/ui/tooltip'
import type { ConversationDetail } from '~/types/conversation'
import type { Attachment, EmailAddress } from '~/types/email'

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
const normalAttachments = computed(() => allAttachments.value.filter((att) => !att.is_inline))

const inlineAttachments = computed(() => allAttachments.value.filter((att) => att.is_inline))

const accountId = computed(() => props.conversation.messages[0]?.account_id)

// AI Notes dialog state
const aiNotesTarget = ref<EmailAddress | null>(null)
const aiNotesOpen = ref(false)

const openAiNotes = (member: EmailAddress) => {
  aiNotesTarget.value = member
  aiNotesOpen.value = true
}
</script>

<template>
  <div class="h-screen border-l border-border bg-surface">
    <div class="border-b border-border px-4 py-3">
      <h2 class="font-semibold text-primary">Conversation Details</h2>
    </div>
    <ScrollArea class="flex-1">
      <div class="space-y-6 px-4 py-4">
        <div>
          <h3 class="mb-3 flex items-center gap-2 text-sm font-semibold tracking-wide uppercase">
            <span>Members</span>
            <Badge
              size="sm"
              variant="background"
              >{{ members.length }}
            </Badge>
          </h3>
          <div class="flex flex-col gap-3">
            <div
              v-for="member in members"
              :key="member.address"
              class="group flex items-center gap-2"
            >
              <RavnAvatar
                :email="member.address"
                :name="member.name"
                class="shrink-0"
                size="lg"
              />
              <div class="min-w-0 flex-1">
                <div class="truncate text-sm font-medium text-primary">
                  {{ member.name || member.address }}
                </div>
                <div
                  v-if="member.name"
                  class="truncate text-xs opacity-60"
                >
                  {{ member.address }}
                </div>
              </div>
              <SimpleTooltip tooltip="Edit AI Notes">
                <Button
                  class="h-6 w-6 shrink-0 text-muted opacity-0 transition-opacity group-hover:opacity-100 hover:text-ai"
                  size="icon"
                  variant="ghost"
                  @click="openAiNotes(member)"
                >
                  <Icon
                    class="h-3.5 w-3.5"
                    name="ravn:raven"
                  />
                </Button>
              </SimpleTooltip>
            </div>
          </div>
        </div>
        <div
          v-if="normalAttachments.length > 0 || inlineAttachments.length > 0"
          class="flex flex-col gap-6"
        >
          <div v-if="normalAttachments.length > 0">
            <h3 class="mb-3 flex items-center gap-2 text-sm font-semibold tracking-wide uppercase">
              <span>Attachments</span>
              <Badge
                size="sm"
                variant="background"
                >{{ normalAttachments.length }}
              </Badge>
            </h3>
            <AttachmentList :attachments="normalAttachments" />
          </div>
          <div v-if="inlineAttachments.length > 0">
            <h3 class="mb-3 flex items-center gap-2 text-sm font-semibold tracking-wide uppercase">
              <span>Inline Attachments</span>
              <Badge
                size="sm"
                variant="background"
                >{{ inlineAttachments.length }}
              </Badge>
            </h3>
            <AttachmentList :attachments="inlineAttachments" />
          </div>
        </div>
      </div>
    </ScrollArea>
  </div>
  <RavnContactAINotesDialog
    v-if="aiNotesTarget"
    v-model:open="aiNotesOpen"
    :email="aiNotesTarget.address"
    :name="aiNotesTarget.name"
  />
</template>
