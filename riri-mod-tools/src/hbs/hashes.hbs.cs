// This file was automatically generated.
// DO NOT EDIT THIS. It will get overwritten if you rebuild {{mod_name}}!
#nullable enable
namespace {{mod_id}}
{
    public partial class Mod
    {
#pragma warning disable CS0414
        {{#each executable_hash}}
        private static ulong {{this.name}} = {{this.hash}};
        {{/each}}
#pragma warning restore CS0414
    }
}