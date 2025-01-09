<task>{{task}}</task>

{{#each files}}
<file_content path="{{this.path}}">
{{this.content}}
</file_content>

{{/each}}