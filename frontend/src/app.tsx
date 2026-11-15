import { useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import { Button, Label, TextArea, TextField } from 'react-aria-components'
import { ArrowRight, Braces, Moon, Sun } from 'lucide-react'
import { ExampleOverview } from './features/assignment/example-overview'
import { PlanResult } from './features/assignment/plan-result'
import { solveProblem } from './shared/api/solve'
import { exampleDefinition } from './shared/examples/assignment'
import { useTheme } from './shared/theme/use-theme'
import './styles.css'
import './features/assignment/assignment.css'

export function App() {
  const { theme, toggleTheme } = useTheme()
  const [definition, setDefinition] = useState(exampleDefinition)
  const solve = useMutation({ mutationFn: solveProblem })
  const showingExample = definition === exampleDefinition

  return (
    <div className="shell" data-theme={theme}>
      <header className="topbar">
        <a className="brand" href="/" aria-label="Orchestrata home">
          <span className="brand-mark"><Braces size={20} aria-hidden /></span>
          <span>orchestrata<span className="brand-dot">.</span></span>
        </a>
        <div className="top-actions">
          <span className="environment">Development preview</span>
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
        <div className="eyebrow"><span className="signal" /> Assignment workspace</div>
        <div className="heading">
          <div>
            <h1>Assign jobs to the right people.</h1>
            <p>Try three jobs and three technicians. See who can take each job without double-booking anyone.</p>
          </div>
        </div>

        <div className="workspace-grid">
          <section className="panel" aria-labelledby="setup-heading">
            <div className="panel-header">
              <div>
                <span className="panel-kicker">1. Set up</span>
                <h2 id="setup-heading">{showingExample ? 'Example: A morning service callout' : 'Your edited setup'}</h2>
              </div>
            </div>

            {showingExample
              ? <ExampleOverview />
              : <p className="custom-setup">This run will use your edited definition instead of the service example.</p>}

            <details className="advanced-panel setup-advanced">
              <summary><Braces size={16} aria-hidden /> Edit problem as JSON</summary>
              <TextField className="editor-field" value={definition} onChange={setDefinition}>
                <Label className="sr-only">Problem JSON</Label>
                <TextArea className="code-editor" spellCheck={false} />
              </TextField>
              <Button
                className="text-button"
                isDisabled={showingExample}
                onPress={() => setDefinition(exampleDefinition)}
              >
                Reset to example
              </Button>
            </details>

            <div className="panel-footer setup-footer">
              <span>{showingExample ? 'No setup needed' : 'Runs your edited definition'}</span>
              <Button className="primary-button" isDisabled={solve.isPending} onPress={() => solve.mutate(definition)}>
                {solve.isPending ? 'Checking assignments' : showingExample ? 'Run example' : 'Run edited setup'}
                <ArrowRight size={16} aria-hidden />
              </Button>
            </div>
          </section>

          <PlanResult
            pending={solve.isPending}
            result={solve.data}
            error={solve.error}
            example={solve.variables === exampleDefinition}
            edited={solve.variables !== undefined && solve.variables !== definition}
          />
        </div>
        <footer className="page-footer">
          Orchestrata / Development preview
          <span>Check the result before using it to schedule real work.</span>
        </footer>
      </main>
    </div>
  )
}
