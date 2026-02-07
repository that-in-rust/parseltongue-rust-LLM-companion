// v151 Bug Reproduction: C# Qualified Names with ::
// This file tests the qualified name bug where :: breaks ISGL1 key parsing
// Expected: Edges should be created, keys should be properly sanitized

using System;
using System.Collections.Generic;
using System.Resources;

namespace MyApp.Services
{
    public class ResourceService
    {
        // Edge 1: Qualified constructor call with ::
        private global::System.Resources.ResourceManager _resources =
            new global::System.Resources.ResourceManager("MyApp.Resources", typeof(ResourceService).Assembly);

        // Edge 2: Fully qualified type reference
        private global::System.Collections.Generic.List<string> _items =
            new global::System.Collections.Generic.List<string>();

        public void LoadResources()
        {
            // Edge 3: Method call with qualified namespace
            var value = global::System.Environment.GetEnvironmentVariable("PATH");

            // Edge 4: Nested namespace access
            var culture = global::System.Globalization.CultureInfo.CurrentCulture;

            // Edge 5: Static method on qualified type
            global::System.Console.WriteLine(value);
        }

        public void ProcessData()
        {
            // Edge 6: Generic type with qualified names
            var dict = new global::System.Collections.Generic.Dictionary<string, object>();

            // Edge 7: Extension method on qualified type
            dict.Add("key", "value");
        }
    }
}
