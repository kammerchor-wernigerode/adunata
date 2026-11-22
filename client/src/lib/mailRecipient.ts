import type { Member } from '@/api/generated/model'

/** The member as a recipient that mail clients accept, quoting the name only when a comma or a double quote would split it. */
export function mailRecipient(member: Pick<Member, 'firstName' | 'lastName' | 'mail'>): string {
  const name = `${member.firstName} ${member.lastName}`
  const quoted = /[",]/.test(name) ? `"${name.replace(/["\\]/g, '\\$&')}"` : name
  return `${quoted} <${member.mail}>`
}
