{{- define "orchestrata.name" -}}
{{- printf "%s-%s" (.root.Release.Name | trunc 52 | trimSuffix "-") .component -}}
{{- end -}}

{{- define "orchestrata.selector" -}}
app.kubernetes.io/name: orchestrata
app.kubernetes.io/instance: {{ .root.Release.Name | quote }}
app.kubernetes.io/component: {{ .component | quote }}
{{- end -}}
