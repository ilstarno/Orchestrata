{{- define "orchestrata-api.fullname" -}}
{{- printf "%s-api" (.Release.Name | trunc 59 | trimSuffix "-") -}}
{{- end -}}

{{- define "orchestrata-api.selector" -}}
app.kubernetes.io/name: orchestrata-api
app.kubernetes.io/instance: {{ .Release.Name | quote }}
{{- end -}}

{{- define "orchestrata-api.image" -}}
{{- $repository := required "set image.repository to your registry/repository" .Values.image.repository -}}
{{- if .Values.image.digest -}}
{{- printf "%s@%s" $repository .Values.image.digest -}}
{{- else -}}
{{- printf "%s:%s" $repository .Values.image.tag -}}
{{- end -}}
{{- end -}}
