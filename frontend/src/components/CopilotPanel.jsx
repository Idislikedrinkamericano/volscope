import { Bot, Send } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { apiJson } from '../lib/api'
import { Panel } from './Primitives'

const STARTER = { role: 'assistant', text: '我只读取当前截面的权威数据。你可以问：IV 贵吗？Gamma 状态如何？这些数据可靠吗？' }

export default function CopilotPanel({ context, disabled, onError }) {
  const [question, setQuestion] = useState('')
  const [messages, setMessages] = useState([STARTER])
  const [loading, setLoading] = useState(false)
  const endRef = useRef(null)

  useEffect(() => { endRef.current?.scrollIntoView({ block: 'nearest' }) }, [messages, loading])

  const submit = async (event) => {
    event.preventDefault()
    const clean = question.trim()
    if (!clean || disabled || loading) return
    setQuestion('')
    setMessages((current) => [...current, { role: 'user', text: clean }])
    setLoading(true)
    try {
      const response = await apiJson('/api/copilot/query', 'POST', { ...context, question: clean })
      setMessages((current) => [...current, { role: 'assistant', text: response.answer, evidence: response.evidence?.filter((item) => item.status === 'available'), snapshotId: response.snapshot_id }])
    } catch (reason) {
      onError?.(reason)
      setMessages((current) => [...current, { role: 'assistant', text: `无法分析当前截面：${reason.message}`, error: true }])
    } finally {
      setLoading(false)
    }
  }

  return <Panel id="copilot" className="copilot-panel" title="Research Copilot" icon={<Bot size={14} />} tools={<span className="copilot-mode">Evidence-only V0</span>}>
    <div className="copilot-shell">
      <div className="copilot-messages" aria-live="polite">
        {messages.map((message, index) => <article key={`${message.role}-${index}`} className={`copilot-message ${message.role} ${message.error ? 'error' : ''}`}>
          <span>{message.role === 'assistant' ? 'COPILOT' : 'YOU'}</span>
          <p>{message.text}</p>
          {message.evidence?.length > 0 && <div className="evidence-strip">{message.evidence.map((item) => <button type="button" key={item.id} title={`${item.source}\n${item.label}: ${item.value} ${item.unit}`}>{item.id} · {item.label}</button>)}</div>}
          {message.snapshotId && <small>Snapshot {message.snapshotId.slice(0, 18)}</small>}
        </article>)}
        {loading && <article className="copilot-message assistant"><span>COPILOT</span><p>正在读取并核对当前证据…</p></article>}
        <div ref={endRef} />
      </div>
      <form className="copilot-input" onSubmit={submit}>
        <input value={question} maxLength={1000} disabled={disabled || loading} onChange={(event) => setQuestion(event.target.value)} placeholder={disabled ? '等待有效的行情截面' : '询问当前截面…'} aria-label="询问 Research Copilot" />
        <button type="submit" disabled={!question.trim() || disabled || loading} title="发送"><Send size={14} /></button>
      </form>
    </div>
  </Panel>
}
