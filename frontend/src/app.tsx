import { useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import { solveProblem } from './shared/api/solve'
import { assignmentExample } from './shared/examples/assignment'
import { useTheme } from './shared/theme/use-theme'
import { Button, Label, TextArea, TextField } from 'react-aria-components'
import { ArrowRight, Braces, Moon, Sun } from 'lucide-react'
import './styles.css'

export function App() {
  const { theme, toggleTheme } = useTheme()
  const [definition, setDefinition] = useState(JSON.stringify(assignmentExample, null, 2))
  const solve = useMutation({ mutationFn: solveProblem })

  return (
    <div className="shell" data-theme={theme}>
      <header className="topbar">
        <a className="brand" href="/" aria-label="Orchestrata home">
          <span className="brand-mark"><Braces size={20} aria-hidden /></span>
          <span>orchestrata<span className="brand-dot">.</span></span>
        </a>
        <div className="top-actions">
          <span className="environment">Development workspace</span>
          <Button
            className="theme-button"
            aria-label={theme === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'}
            onPress={toggleTheme}
          >
            {theme === 'dark' ? <Sun size={19} aria-hidden /> : <Moon size={19} aria-hidden />}
          </Button>
        </div>
      </header>

      <main className="content">
        <div className="eyebrow"><span className="signal" /> Optimization workspace / Reference engine</div>
        <div className="heading">
          <div>
            <h1>Define the problem.<br /><span>Explore the solution.</span></h1>
            <p>Describe decisions, value ranges, and constraints. The reference engine evaluates assignments and returns the best result within its search limit.</p>
          </div>
          <div className="engine-tag">FINITE DOMAIN <span>•</span> v0.1</div>
        </div>

        <div className="workspace-grid">
          <section className="panel">
            <div className="panel-header">
              <div><span className="panel-kicker">01 / INPUT</span><h2>Problem definition</h2></div>
              <span className="file-label">problem.json</span>
            </div>
            <TextField className="editor-field" value={definition} onChange={setDefinition}>
              <Label className="sr-only">Problem JSON</Label>
              <TextArea className="code-editor" spellCheck={false} />
            </TextField>
            <div className="panel-footer">
              <span>JSON · Integer domains</span>
              <Button className="primary-button" isDisabled={solve.isPending} onPress={() => solve.mutate(definition)}>
                {solve.isPending ? 'Evaluating' : 'Run solver'} <ArrowRight size={16} aria-hidden />
              </Button>
            </div>
          </section>

          <section className="panel result-panel">
            <div className="panel-header">
              <div><span className="panel-kicker">02 / OUTPUT</span><h2>Evaluation result</h2></div>
              <span className="result-status">{solve.isPending ? 'Running' : solve.isSuccess ? 'Complete' : 'Awaiting run'}</span>
            </div>
            <div className="result-content" aria-live="polite">
              {solve.isError ? <p className="error">{solve.error.message}</p> : null}
              {solve.isSuccess ? <pre className="result-json">{JSON.stringify(solve.data, null, 2)}</pre> : null}
              {!solve.isError && !solve.isSuccess ? (
                <div className="empty-state">
                  <div className="empty-icon"><Braces size={28} strokeWidth={1.5} aria-hidden /></div>
                  <h3>Ready to evaluate</h3>
                  <p>Run the example to see assignments, hard and soft scores, and the number of evaluated candidates.</p>
                </div>
              ) : null}
            </div>
            <div className="panel-footer muted">Reference solver · Maximum 100,000 combinations</div>
          </section>
        </div>
        <footer className="page-footer">Orchestrata / Development preview <span>Results are not production scheduling recommendations.</span></footer>
      </main>
    </div>
  )
}
